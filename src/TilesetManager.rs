use std::fs;

use bespoke_engine::{binding::UniformBinding, texture::Texture};
use cgmath::Vector2;
use image::{GenericImageView, ImageBuffer, Pixel, Rgb, Rgba};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::physics::boxCollider::BoxCollider;

#[derive(Serialize, Deserialize)]
struct Tilemap {
    mapHeight: u32,
    mapWidth: u32,
    tileSize: u32,
    layers: Vec<Layer>,
}

#[derive(Serialize, Deserialize)]
struct Layer {
    collider: bool,
    name: String,
    tiles: Vec<Tile>,
}

#[derive(Serialize, Deserialize)]
struct Tile {
    id: String,
    x: u32,
    y: u32,
}


pub struct TilesetManager {
    pub colliders: Vec<BoxCollider>,
}

impl TilesetManager {
    pub fn new(json_path: &str, map_width: u32) -> Self {
        //read the json of the tilemap
        //using https://www.spritefusion.com/editor as tilemap editor
        let json_contents = fs::read_to_string(json_path).expect("Couldn't read JSON");
        let data : Tilemap = serde_json::from_str(&json_contents).unwrap();
        //create a new image to write tile data to
        let mut newImage = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(data.mapWidth * data.tileSize, data.mapHeight * data.tileSize);
        let tileset = image::open("src/res/spritesheet.png").unwrap();
        let tilesetWidth = tileset.width() / data.tileSize;

        let mut colliders = Vec::new();

        
        
        //loop through all of the tiles in the tile map, and then add them to the new image
        for layer in data.layers.into_iter().rev() {
            for tile in layer.tiles {
                //the position that we are at in the image
                let xPos = tile.x;
                let yPos = tile.y;

                //the position that we are at in the tileset
                let tileX = tile.id.parse::<u32>().unwrap() % tilesetWidth;
                let tileY = (tile.id.parse::<u32>().unwrap() - (tile.id.parse::<u32>().unwrap() % tilesetWidth)) / tilesetWidth;

                for x in 0..data.tileSize {
                    for y in 0..data.tileSize {
                        let tileset_color = tileset.get_pixel(tileX * data.tileSize + x, tileY * data.tileSize + y);
                        if (tileset_color.0[3] == 255)
                        {
                            newImage.put_pixel(xPos * data.tileSize + x, yPos * data.tileSize + y, tileset_color);
                        }
                        
                    }
                }

                //create a collider for each tile in the tile map
                let scale_factor = map_width as f32 / (data.mapWidth * data.tileSize) as f32;
                let coll_pos = Vector2::new(-(xPos as f32 * data.tileSize as f32 * scale_factor - (data.mapWidth as f32 * data.tileSize as f32 * scale_factor / 2.0) as f32) - data.tileSize as f32 * scale_factor / 2.0, -(yPos as f32 * data.tileSize as f32 * scale_factor  - (data.mapHeight as f32 * data.tileSize as f32 * scale_factor / 2.0) as f32) - data.tileSize as f32 * scale_factor / 2.0);
                let coll = BoxCollider::new(coll_pos, Vector2::new(data.tileSize as f32 * scale_factor, data.tileSize as f32 * scale_factor));
                colliders.push(coll);
            }
        }

        newImage.save("src/res/output.png").unwrap();
        

        Self {
            colliders,
        }
    }
}