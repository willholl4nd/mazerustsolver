use image::{Rgb, GenericImage, GenericImageView, ImageBuffer, open, DynamicImage};
use image::io::Reader;
use std::io::Cursor;
use std::env;
use std::fmt::{Display, Formatter, Result};
use std::time::Instant;
use std::vec::Vec;
use sqrt;

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
    let north: Vec<(usize, &Rgb<u8>)> = maze.pixels().enumerate().filter(|(index, pixel)| *index < maze.width() as usize).collect();
    let north_count = north.len();

    let south: Vec<(usize, &Rgb<u8>)> = maze.pixels().enumerate().filter(|(index, pixel)| *index as u32 >= (maze.height()-1) * maze.width()).collect();
    let south_count = south.len();

    let mut west: Vec<(usize, &Rgb<u8>)> = maze.pixels().enumerate().filter(|(index, pixel)| *index % maze.width() as usize == 0).collect();
    west.remove(0); //Removed TL corner contained in north
    west.remove(west.len()-1); //Removed BL corner contained in south
    let west_count = west.len(); 

    let mut east: Vec<(usize, &Rgb<u8>)> = maze.pixels().enumerate().filter(|(index, pixel)| *index as u32 % maze.width() == maze.width()-1).collect();
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
    let col: u32 = (index - (index / width as usize) * height as usize) as u32;
    println!("{:?}", (row,col));
    (row, col)
}



/**
 * Convert a two dimensional (row, column) pair to a one 
 * dimensional index to array a flat array
 */
fn two_to_one_D(row: u32, col: u32, width: u32) -> usize {
    let index: usize = (row * width + col) as usize;
    println!("{}", index);
    index
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

    //Parse image for nodes
    
}
