use std::fs;

use bespoke_engine::{binding::UniformBinding, texture::Texture};
use image::{GenericImageView, ImageBuffer, Pixel, Rgb, Rgba};
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    // tileset_image: UniformBinding<Texture>,
    // rendered_image: UniformBinding<Texture>,
}

impl TilesetManager {
    pub fn new(json_path: &str) -> Self {
        let json_contents = fs::read_to_string(json_path).expect("Couldn't read JSON");
        let data : Tilemap = serde_json::from_str(&json_contents).unwrap();

        let mut newImage = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(data.mapWidth * data.tileSize, data.mapHeight * data.tileSize);
        let tileset = image::open("src/res/spritesheet.png").unwrap();
        let tilesetWidth = tileset.width() / data.tileSize;
        let tilesetHeight = tileset.height() / data.tileSize;

        
        

        for layer in data.layers {
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
                        newImage.put_pixel(xPos * data.tileSize + x, yPos * data.tileSize + y, tileset_color);
                    }
                }
            }
        }

        newImage.save("src/res/output.png").unwrap();
        

        Self {

        }
    }
}