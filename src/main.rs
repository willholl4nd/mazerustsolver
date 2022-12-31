use image::{Rgb, ImageBuffer, DynamicImage};
use image::io::Reader;
use std::env;
use std::mem::size_of;
use std::time::Instant;
use std::vec::Vec;
use std::fmt::{Debug, Formatter, Error};
use sqrt;
use indicatif::ProgressBar;

struct Grid {
    width: u32, 
    height: u32,
    grid: Vec<Option<Node>>,
    
    //(row, col)
    start: (u32, u32),
    end: (u32, u32),
}

impl Grid {
    pub fn new(width: u32, height: u32, start: (u32, u32), end: (u32, u32)) -> Self {
        let mut grid = Vec::<Option<Node>>::new();
        for _ in 0..width*height {
            grid.push(None);
        }

        Grid {
            width, 
            height,
            grid,
            start,
            end
        }
    }

    pub fn init(&mut self, node_positions: Vec<(u32, u32)>) {
        let mut bar = ProgressBar::new(node_positions.len() as u64);
        node_positions.iter().for_each(|(row,col)| {
            bar.inc(1);
            self.put(Some(Node::new(*row, *col)), *row, *col);
        });
        bar.finish();
    }

    pub fn get(&self, row: u32, col: u32) -> &Option<Node> {
        let index: usize = two_to_one_D(row, col, self.width, self.height);
        let node = self.grid.get(index);
        return match node {
            None => {
                &None
            },
            Some(&_) => {
                node.unwrap()
            }
        };
    }

    pub fn get_mut(&mut self, row: u32, col: u32) -> &Option<Node> {
        let index: usize = two_to_one_D(row, col, self.width, self.height);
        let node = self.grid.get_mut(index);
        return match node {
            None => {
                &None
            },
            Some(&mut _) => {
                node.unwrap()
            }
        };
    }
    pub fn put(&mut self, node: Option<Node>, row: u32, col: u32) -> Option<Node> {
        let index: usize = two_to_one_D(row, col, self.width, self.height);
        std::mem::replace(&mut self.grid[index], node)
    }

    pub fn connect_horiz(&mut self) {

        let mut currentNode: &Option<Node>; //The current node we are considering
        let mut has_east: &Option<Box<(u32, u32)>>;

        //Loop through entire maze
        for row in 0..self.height-1 {
            for col in 0..self.width-1 {
                let mut node: &Option<Node> = self.get_mut(row, col); //The comparison node
                match node {
                    &None => { },
                    Some(n) => {
                        has_east = &n.e_node;

                        //if has_east == &None && 
                    }
                }
            }
        }
    }

    pub fn connect_vertical(&mut self) {

    }

    pub fn print(&self) {
        println!("Length of grid: {}", self.grid.len());
        for row in 0..self.height {
            for col in 0..self.width {
                print!("{:?} ", self.get(row, col));
            }
            println!("");
        }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

struct Node {
    //(row, col)
    n_node: Option<Box<(u32, u32)>>,
    e_node: Option<Box<(u32, u32)>>,
    s_node: Option<Box<(u32, u32)>>,
    w_node: Option<Box<(u32, u32)>>,
    came_from_node: Option<Box<(u32, u32)>>,

    is_start_node: bool,
    is_end_node: bool,
    location: (u32, u32),
}

impl Debug for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> { 
        f.debug_struct("Node").field("row", &self.location.0)
            .field("col", &self.location.1).finish()
    }
}

impl Node {
    pub fn new(row: u32, col: u32) -> Self {
        Node {
            n_node: None,
            e_node: None,
            s_node: None,
            w_node: None,
            came_from_node: None,
            is_start_node: false,
            is_end_node: false,
            location: (row, col)
        }
    }
}

/**
 * Does a bunch of checks to make sure the image border is correct
 * Checks:
 * 1. Only two colors exist in border
 * 2. One of the two colors appears only twice in border (start, end)
 * 3. Color appearing twice doesn't appear in corner of maze image
 *
 * Returns:
 * ((start x dimensions, start y dimensions), (end x dimensions, end y dimensions),
 * path color, background color)
 */
fn start_end_detect(maze: &ImageBuffer<Rgb<u8>,Vec<u8>>) -> ((u32,u32), (u32,u32), Rgb<u8>, Rgb<u8>) {
    let north: Vec<(usize, &Rgb<u8>)> = maze.pixels().enumerate().filter(|(index, _)| *index < maze.width() as usize).collect();
    let north_count = north.len();

    let south: Vec<(usize, &Rgb<u8>)> = maze.pixels().enumerate().filter(|(index, _)| *index as u32 >= (maze.height()-1) * maze.width()).collect();
    let south_count = south.len();

    let mut west: Vec<(usize, &Rgb<u8>)> = maze.pixels().enumerate().filter(|(index, _)| *index % maze.width() as usize == 0).collect();
    west.remove(0); //Removed TL corner contained in north
    west.remove(west.len()-1); //Removed BL corner contained in south
    let west_count = west.len(); 

    let mut east: Vec<(usize, &Rgb<u8>)> = maze.pixels().enumerate().filter(|(index, _)| *index as u32 % maze.width() == maze.width()-1).collect();
    east.remove(0); //Removed TR corner contained in north
    east.remove(east.len()-1); //Removed BR corner contained in south
    let east_count = east.len();

    let mut border: Vec<(usize, &Rgb<u8>)> = Vec::new();
    border.extend(north.iter());
    border.extend(east.iter());
    border.extend(south.iter());
    border.extend(west.iter());
    println!("north size: {}, east size: {}, south size: {}, west size: {}",
             north_count, east_count, south_count, west_count);

    let mut set: Vec<&Rgb<u8>> = Vec::new(); //Holds all colors in border
    border.iter().for_each(|(_,pixel)| if !set.contains(pixel) { set.push(pixel.clone()) }); //Find all colors

    //Check that only two colors exist in the border
    if set.len() != 2 {
        panic!("ERROR: Too many colors used to make maze. Required that you use only 2 colors");
    }

    let first_color_nodes: Vec<_> = border.iter().filter(|(_,pixel)| pixel == set.get(0).unwrap()).collect();
    let second_color_nodes: Vec<_> = border.iter().filter(|(_,pixel)| pixel == set.get(1).unwrap()).collect();
    let first_color_count = first_color_nodes.len();
    let second_color_count = second_color_nodes.len();
    println!("First color {:?} has appeared {} times in the border", set.get(0).unwrap(), first_color_count);
    println!("Second color {:?} has appeared {} times in the border", set.get(1).unwrap(), second_color_count);

    //Check for which color will be the path and which will be the background
    let path_color: Option<Rgb<u8>>;
    let back_color: Option<Rgb<u8>>;
    let start_end_indices: Vec<usize>;
    if first_color_count == 2 {
        path_color = Some(*set.get(0).unwrap().clone());
        back_color = Some(*set.get(1).unwrap().clone());
        start_end_indices = first_color_nodes.iter().map(|(i, _)| *i).collect();
    } else if second_color_count == 2 {
        back_color = Some(*set.get(0).unwrap().clone());
        path_color = Some(*set.get(1).unwrap().clone());
        start_end_indices = second_color_nodes.iter().map(|(i, _)| *i).collect();
    } else {
        eprintln!("ERROR: Neither of the colors had a count of two: \n\t{:?}",
                  if first_color_count < second_color_count { first_color_nodes } else { second_color_nodes });
        panic!();
    }
    println!("Path color: {:?}, background color: {:?}", path_color, back_color);

    //Check if the start or end node is in the corner of the maze
    println!("Indices for starting and ending nodes: {:?}", start_end_indices);
    let TL = 0;
    let TR = maze.width()-1;
    let BL = (maze.height()-1) * maze.width();
    let BR = maze.height() * maze.width() - 1;
    let unacceptable_indices = [TL, TR, BL, BR];
    start_end_indices.iter().for_each(|i| 
                                      if unacceptable_indices.contains(&(*i as u32)) { 
                                          eprintln!("ERROR: A start or end position is in a corner\n\
                                                    \tUnacceptable indices: {:?}", unacceptable_indices);
                                          panic!(); 
                                      } 
                                     );

    //All checks have been made and now we can continue to return locations for 
    //start and end nodes
    let first_index = start_end_indices.get(0).unwrap();
    let (mut first_dim_x, mut first_dim_y): (u32, u32) = one_to_two_D(*first_index, maze.width(), maze.height());
    println!("First node is at x: {}, y: {}", first_dim_x, first_dim_y);
    let second_index = start_end_indices.get(1).unwrap();
    let (mut second_dim_x, mut second_dim_y): (u32, u32) = one_to_two_D(*second_index, maze.width(), maze.height());

    println!("Second node is at x: {}, y: {}", second_dim_x, second_dim_y);
    let first_distance = sqrt::sqrt((first_dim_x * first_dim_x + first_dim_y * first_dim_y) as f64);
    let second_distance = sqrt::sqrt((second_dim_x * second_dim_x + second_dim_y * second_dim_y) as f64);
    println!("First node is {} units from origin\nSecond node is {} units from origin", first_distance, second_distance);

    //Node closer to the origin is the starting node 
    if first_distance > second_distance {
        ((first_dim_x, first_dim_y), (second_dim_x, second_dim_y)) = ((second_dim_x, second_dim_y), (first_dim_x, first_dim_y));
    }

    ((first_dim_x as u32, first_dim_y as u32), (second_dim_x as u32, second_dim_y as u32), path_color.unwrap(), back_color.unwrap())
}

/**
 * Checks all pixels in image to make sure they are one of two colors
 *
 * Returns false if there is more that 2 colors in image
 */
fn perform_image_check(maze: &ImageBuffer<Rgb<u8>, Vec<u8>>, path_color: &Rgb<u8>, back_color: &Rgb<u8>) -> bool {
    for pixel in maze.pixels().into_iter() {
        if *pixel == *path_color || *pixel == *back_color {
            continue;
        } 
        return false;
    }
    true
}

/**
 * Convert a one dimensional index to a (row, column) pair 
 * to address a two dimensional array
 */
fn one_to_two_D(index: usize, width: u32, height: u32) -> (u32, u32) {
    let row: u32 = (index / width as usize) as u32;
    let col: u32 = (index - row as usize * height as usize) as u32;
    if row >= height || col >= width {
        panic!("ERROR: index argument is outside of dimensions");
    }
    (row, col)
}



/**
 * Convert a two dimensional (row, column) pair to a one 
 * dimensional index to array a flat array
 */
fn two_to_one_D(row: u32, col: u32, width: u32, height: u32) -> usize {
    let index: usize = (row * width + col) as usize;
    if index >= (width * height) as usize {
        panic!("ERROR: row and col arguments are outside of dimensions");
    }
    index
}

fn find_nodes(maze: &ImageBuffer<Rgb<u8>,Vec<u8>>, path_color: &Rgb<u8>) -> Vec<(u32, u32)> {
    let mut ret: Vec<(u32, u32)> = Vec::new();
    let width: u32 = maze.width();
    let height: u32 = maze.height();
    let mut bar = ProgressBar::new((height*width) as u64);

    //Loop over all pixels
    for row in 0..height {
        for col in 0..width {
            if is_node(maze, path_color, col, row) {
                ret.push((row, col));
            } 
        }
        bar.inc(width as u64);
    }
    bar.finish();
    ret
}

fn is_node(maze: &ImageBuffer<Rgb<u8>,Vec<u8>>, path_color: &Rgb<u8>, col: u32, row: u32) -> bool {
    //Check if cords are a path tile
    if maze.get_pixel(col, row) != path_color {
        return false;
    }

    let mut count: u8 = 0;
    let line_ew: bool;
    let line_ns: bool;
    let width = maze.width();
    let height = maze.height();

    let mut north: Option<&Rgb<u8>> = None;
    let mut east: Option<&Rgb<u8>> = None;
    let mut south: Option<&Rgb<u8>> = None;
    let mut west: Option<&Rgb<u8>> = None;

    let mut nb: bool = false;
    let mut eb: bool = false;
    let mut sb: bool = false;
    let mut wb: bool = false;

    if in_bounds(width, height, col as i64 + 1, row as i64) && maze.get_pixel(col+1, row) == path_color {
        east = Some(maze.get_pixel(col+1, row));
        eb = true;
        count += 1;
    }
    if in_bounds(width, height, col as i64 - 1, row as i64) && maze.get_pixel(col-1, row) == path_color {
        west = Some(maze.get_pixel(col-1, row));
        wb = true;
        count += 1;
    }
    if in_bounds(width, height, col as i64, row as i64 + 1) && maze.get_pixel(col, row+1) == path_color {
        south = Some(maze.get_pixel(col, row+1));
        sb = true;
        count += 1;
    }
    if in_bounds(width, height, col as i64, row as i64 - 1) && maze.get_pixel(col, row-1) == path_color {
        north = Some(maze.get_pixel(col, row-1));
        nb = true;
        count += 1;
    }

    line_ew = eb && east.unwrap() == path_color && wb && west.unwrap() == path_color;
    line_ns = nb && north.unwrap() == path_color && sb && south.unwrap() == path_color;

    count == 1 || (count == 2 && !(line_ns ||  line_ew)) || count == 3 || count == 4
}

fn in_bounds(width: u32, height: u32, col: i64, row: i64) -> bool {
    col >= 0 && col < width as i64 && row >= 0 && row < height as i64
}

fn main() {
    let argv: Vec<String> = env::args().collect();
    let argc = argv.len();
    if argc < 2 {
        eprintln!("Must provide an argument for the file path");
        return;
    }

    let file_name: &String = argv.get(1).unwrap();
    println!("The file name passed in {file_name}");

    //Reader for image with no limits on size
    let mut reader = Reader::open(file_name).unwrap();
    reader.no_limits();
    let maze_data: DynamicImage = reader.decode().unwrap();

    //Load image
    let maze: ImageBuffer<Rgb<u8>, Vec<u8>> = maze_data.into_rgb8();
    println!("File has dimensions {:?}", maze.dimensions());

    //Parse image file for terminal nodes nodes and colors 
    let ((start_col, start_row), (end_col, end_row), path_color, back_color) = start_end_detect(&maze);
    if !perform_image_check(&maze, &path_color, &back_color) {
        panic!("Image contains more than 2 colors");
    }

    //Convert image into node objects
    println!("Creating grid object");
    let mut nodes: Grid = Grid::new(maze.width(), maze.height(), (start_row, start_col), (end_row, end_col));

    println!("Finding all nodes in maze");
    let node_positions = find_nodes(&maze, &path_color);
    nodes.init(node_positions);
    nodes.connect_horiz();
    nodes.connect_vertical();

    todo!("Think of how you could use incremental loading to get around high memory usage");

    //Parse image for nodes
}
