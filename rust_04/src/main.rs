use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};
use std::env;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::thread;
use std::time::Duration;

type Cost = u32;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
        print_help();
        return Ok(());
    }

    let mut generate = None;
    let mut output = None;
    let mut visualize = false;
    let mut both = false;
    let mut animate = false;
    let mut map_file = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--generate" => {
                i += 1;
                if i < args.len() {
                    generate = Some(args[i].clone());
                }
            }
            "--output" => {
                i += 1;
                if i < args.len() {
                    output = Some(args[i].clone());
                }
            }
            "--visualize" => visualize = true,
            "--both" => both = true,
            "--animate" => animate = true,
            arg if !arg.starts_with('-') => map_file = Some(arg.to_string()),
            _ => {}
        }
        i += 1;
    }

    let mut grid: Option<Vec<Vec<u8>>> = None;

    if let Some(mut gen_str) = generate {
        gen_str = gen_str.replace('*', "x");
        let parts: Vec<usize> = gen_str.split('x').map(|s| s.trim().parse().unwrap()).collect();
        let rows = parts[0];
        let cols = parts[1];
        grid = Some(generate_map(rows, cols));
        if let Some(out_file) = output {
            save_map(&grid.as_ref().unwrap(), &out_file)?;
            println!("Generated map saved to {}.", out_file);
        }
    } else if let Some(file) = map_file {
        grid = Some(read_map(&file)?);
    }

    if let Some(ref g) = grid {
        let (min_cost, min_path, max_cost, max_path) = compute_paths(g);
        println!("Grid size: {}x{}", g.len(), g[0].len());
        println!("Start: (0,0) = 00");
        println!("End: ({}, {}) = FF", g.len() - 1, g[0].len() - 1);

        println!("\nMINIMUM COST PATH:");
        println!("Total cost: 0x{:X} ({} decimal)", min_cost, min_cost);
        println!("Path length: {} steps", min_path.len() - 1);
        print_path(&min_path);
        println!("\nStep-by-step costs:");
        print_step_costs(&min_path, g);

        println!("\nMAXIMUM COST PATH:");
        println!("Total cost: 0x{:X} ({} decimal)", max_cost, max_cost);
        println!("Path length: {} steps", max_path.len() - 1);
        print_path(&max_path);
        println!("\nStep-by-step costs:");
        print_step_costs(&max_path, g);
    }

    if visualize {
        if let Some(ref g) = grid {
            println!("\nHEXADECIMAL GRID (rainbow gradient):");
            print_grid_colored(g, &vec![], "");
            if both {
                let (_, min_path, _, max_path) = compute_paths(g);
                println!("\nMIN COST PATH (shown in WHITE):");
                print_grid_colored(g, &min_path, "white");
                println!("\nMAX COST PATH (shown in RED):");
                print_grid_colored(g, &max_path, "red");
            }
        }
    }

    if animate {
        if let Some(ref g) = grid {
            animate_min(g);
        }
    }

    Ok(())
}

fn print_help() {
    println!("cargo run --help");
    println!("Usage: hexpath [OPTIONS]");
    println!("Find min/max cost paths in hexadecimal grid");
    println!("\nArguments:");
    println!("Map file (hex values, space separated)");
    println!("\nMap format:");
    println!("- Each cell: 00-FF (hexadecimal)");
    println!("- Start: top-left (must be 00)");
    println!("- End: bottom-right (must be FF)");
    println!("- Moves: down, right");
    println!("\nOptions:");
    println!("--generate Generate random map (e.g., 8x4, 10x10)");
    println!("--output Save generated map to file");
    println!("--visualize Show colored map");
    println!("--both Show both min and max paths");
    println!("--animate Animate pathfinding");
    println!("-h, --help");
}

fn generate_map(rows: usize, cols: usize) -> Vec<Vec<u8>> {
    let inc_row = 12u32;
    let inc_col = 21u32;
    let mut grid = vec![vec![0u8; cols]; rows];
    for i in 0..rows {
        for j in 0..cols {
            let val = ((i as u32 * inc_row + j as u32 * inc_col) % 256) as u8;
            grid[i][j] = val;
        }
    }
    grid[0][0] = 0;
    grid[rows - 1][cols - 1] = 255;
    grid
}

fn save_map(grid: &Vec<Vec<u8>>, file: &str) -> io::Result<()> {
    let mut f = File::create(file)?;
    for row in grid {
        for &v in row {
            write!(f, "{:02X} ", v)?;
        }
        writeln!(f)?;
    }
    Ok(())
}

fn read_map(file: &str) -> io::Result<Vec<Vec<u8>>> {
    let f = File::open(file)?;
    let reader = io::BufReader::new(f);
    let mut grid = vec![];
    for line in reader.lines() {
        let line = line?;
        let row: Vec<u8> = line.split_whitespace().filter(|s| !s.is_empty()).map(|s| u8::from_str_radix(s, 16).unwrap()).collect();
        if !row.is_empty() {
            grid.push(row);
        }
    }
    Ok(grid)
}

fn compute_paths(grid: &Vec<Vec<u8>>) -> (Cost, Vec<(usize, usize)>, Cost, Vec<(usize, usize)>) {
    let directions = vec![(1isize, 0), (0, 1)]; 
    let (min_cost, min_parent) = dijkstra_min(grid, &directions);
    let min_path = reconstruct_path(&min_parent, grid.len() - 1, grid[0].len() - 1);
    let (max_cost, max_parent) = dijkstra_max(grid, &directions);
    let max_path = reconstruct_path(&max_parent, grid.len() - 1, grid[0].len() - 1);
    (min_cost, min_path, max_cost, max_path)
}

fn dijkstra_min(grid: &Vec<Vec<u8>>, dirs: &Vec<(isize, isize)>) -> (Cost, Vec<Vec<Option<(usize, usize)>>> ) {
    let rows = grid.len();
    let cols = grid[0].len();
    let mut dist = vec![vec![u32::MAX; cols]; rows];
    dist[0][0] = grid[0][0] as u32;
    let mut pq = BinaryHeap::new();
    pq.push(Reverse((dist[0][0], 0, 0)));
    let mut parent = vec![vec![None; cols]; rows];
    while let Some(Reverse((cost, i, j))) = pq.pop() {
        if cost > dist[i][j] { continue; }
        for &(di, dj) in dirs {
            let ni = i as isize + di;
            let nj = j as isize + dj;
            if ni >= 0 && ni < rows as isize && nj >= 0 && nj < cols as isize {
                let ni = ni as usize;
                let nj = nj as usize;
                let new_cost = cost + grid[ni][nj] as u32;
                if new_cost < dist[ni][nj] {
                    dist[ni][nj] = new_cost;
                    parent[ni][nj] = Some((i, j));
                    pq.push(Reverse((new_cost, ni, nj)));
                }
            }
        }
    }
    (dist[rows - 1][cols - 1], parent)
}

fn dijkstra_max(grid: &Vec<Vec<u8>>, dirs: &Vec<(isize, isize)>) -> (Cost, Vec<Vec<Option<(usize, usize)>>> ) {
    let rows = grid.len();
    let cols = grid[0].len();
    let mut dist = vec![vec![0; cols]; rows];
    dist[0][0] = grid[0][0] as u32;
    let mut pq = BinaryHeap::new();
    pq.push((dist[0][0], 0, 0));
    let mut parent = vec![vec![None; cols]; rows];
    while let Some((cost, i, j)) = pq.pop() {
        if cost < dist[i][j] { continue; }
        for &(di, dj) in dirs {
            let ni = i as isize + di;
            let nj = j as isize + dj;
            if ni >= 0 && ni < rows as isize && nj >= 0 && nj < cols as isize {
                let ni = ni as usize;
                let nj = nj as usize;
                let new_cost = cost + grid[ni][nj] as u32;
                if new_cost > dist[ni][nj] {
                    dist[ni][nj] = new_cost;
                    parent[ni][nj] = Some((i, j));
                    pq.push((new_cost, ni, nj));
                }
            }
        }
    }
    (dist[rows - 1][cols - 1], parent)
}

fn reconstruct_path(parent: &Vec<Vec<Option<(usize, usize)>>>, mut i: usize, mut j: usize) -> Vec<(usize, usize)> {
    let mut path = vec![(i, j)];
    while let Some((pi, pj)) = parent[i][j] {
        path.push((pi, pj));
        i = pi;
        j = pj;
    }
    path.reverse();
    path
}

fn print_path(path: &Vec<(usize, usize)>) {
    for (k, &(i, j)) in path.iter().enumerate() {
        if k > 0 {
            print!("-");
        }
        print!("({}, {})", i, j);
    }
    println!();
}

fn print_step_costs(path: &Vec<(usize, usize)>, grid: &Vec<Vec<u8>>) {
    let mut first = true;
    for &(i, j) in path {
        if first {
            println!("Start 0x{:02X} ({}, {})", grid[i][j], i, j);
            first = false;
        } else {
            println!(" - 0x{:02X} ({}, {})", grid[i][j], i, j);
        }
    }
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (rp, gp, bp) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    (((rp + m) * 255.0) as u8, ((gp + m) * 255.0) as u8, ((bp + m) * 255.0) as u8)
}

fn print_grid_colored(grid: &Vec<Vec<u8>>, path: &Vec<(usize, usize)>, path_color: &str) {
    let path_set: HashSet<(usize, usize)> = path.iter().cloned().collect();
    for i in 0..grid.len() {
        for j in 0..grid[0].len() {
            let v = grid[i][j];
            if path_set.contains(&(i, j)) {
                let color_code = match path_color {
                    "white" => "\x1b[37m",
                    "red" => "\x1b[31m",
                    _ => "\x1b[0m",
                };
                print!("{}{:02X}\x1b[0m ", color_code, v);
            } else {
                let hue = (v as f32 / 255.0) * 360.0;
                let (r, g, b) = hsl_to_rgb(hue, 1.0, 0.5);
                print!("\x1b[38;2{};{};{}m{:02X}\x1b[0m ", r, g, b, v);
            }
        }
        println!();
    }
}

fn animate_min(grid: &Vec<Vec<u8>>) {
    let directions = vec![(1isize, 0), (0, 1)];
    let rows = grid.len();
    let cols = grid[0].len();
    let mut dist = vec![vec![u32::MAX; cols]; rows];
    dist[0][0] = grid[0][0] as u32;
    let mut pq = BinaryHeap::new();
    pq.push(Reverse((dist[0][0], 0, 0)));
    let mut step = 1;
    while let Some(Reverse((cost, i, j))) = pq.pop() {
        println!("Step {}: Exploring ({}, {}) - cost: {}", step, i, j, cost);
        for x in 0..rows {
            for y in 0..cols {
                if x == i && y == j {
                    print!("[*] ");
                } else if dist[x][y] != u32::MAX {
                    print!("[.] ");
                } else {
                    print!("[ ] ");
                }
            }
            println!();
        }
        println!();
        step += 1;
        thread::sleep(Duration::from_millis(500));
        if i == rows - 1 && j == cols - 1 {
            println!("Path found!");
            break;
        }
        for &(di, dj) in &directions {
            let ni = (i as isize + di) as usize;
            let nj = (j as isize + dj) as usize;
            if ni < rows && nj < cols {
                let new_cost = cost + grid[ni][nj] as u32;
                if new_cost < dist[ni][nj] {
                    dist[ni][nj] = new_cost;
                    pq.push(Reverse((new_cost, ni, nj)));
                }
            }
        }
    }
}