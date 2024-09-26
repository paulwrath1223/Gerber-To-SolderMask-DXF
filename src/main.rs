use std::fs::File;
use std::io::BufReader;

use gerber_parser::gerber_doc::GerberDoc;
use gerber_parser::parser::parse_gerber;
use gerber_types::{Aperture, Command, Coordinates, DCode, GCode, InterpolationMode, Operation, Unit};
use gerber_types::{CoordinateOffset, FunctionCode};
use dxf;


fn main() {
    let file = File::open("test_files/solderpaste_top.gbr").expect("failed to open");
    let reader = BufReader::new(file);
    let gerber_doc: GerberDoc = parse_gerber(reader);
    println!("units: {:?}\nname: {:?}", gerber_doc.units, gerber_doc.image_name);
    
    let units: dxf::enums::Units = match gerber_doc.units{
        Some(units) => {
            match units{
                Unit::Inches => {dxf::enums::Units::Inches}
                Unit::Millimeters => {dxf::enums::Units::Millimeters}
            }
        }
        None => {dxf::enums::Units::Unitless}
    };

    let mut drawing = dxf::Drawing::new();
    drawing.header.default_drawing_units = units;
    
    let mut current_aperture: Option<i32> = None;
    
    for (id, aperture) in &gerber_doc.apertures{
        let mut block = dxf::Block {
            name: id.to_string(),
            ..Default::default()
        };

        //
        // ...and populate it with entities
        //
        match aperture{
            Aperture::Circle(circle) => {
                block.entities.push(dxf::entities::Entity {
                    common: Default::default(),
                    specific: dxf::entities::EntityType::Circle(
                        dxf::entities::Circle::new(
                            dxf::Point::new(0.0, 0.0, 0.0), circle.diameter
                        )
                    )
                });
            }
            Aperture::Rectangle(rectangle) => {}
            Aperture::Obround(obround) => {}
            Aperture::Polygon(polygon) => {}
            Aperture::Other(other) => {}
        }
        

        //
        // add the block to the drawing
        //
        drawing.add_block(block);
    }

    for command in &gerber_doc.commands{
        match command{
            Command::FunctionCode(fc) => {
                match fc{
                    FunctionCode::DCode(dc) => {
                        match dc{
                            DCode::Operation(operation) => {
                                match operation {
                                    Operation::Interpolate(_, _) => {}
                                    Operation::Move(_) => {}
                                    Operation::Flash(coords) => {
                                        flash_aperture_at_coords(&mut drawing, &gerber_doc, current_aperture, coords);
                                    }
                                }
                            }
                            DCode::SelectAperture(aperture_id) => {
                                current_aperture = Some(*aperture_id);
                            }
                        }
                    }
                    FunctionCode::GCode(gc) => {
                        match gc{
                            GCode::InterpolationMode(_) => {}
                            GCode::RegionMode(_) => {}
                            GCode::QuadrantMode(_) => {}
                            GCode::Comment(_) => {}
                        }
                    }
                    FunctionCode::MCode(_) => {}
                }
            }
            Command::ExtendedCode(ec) => {}
        }
    }
}


fn flash_aperture_at_coords(drawing: &mut dxf::Drawing, gerber_doc: &GerberDoc, aperture_id: Option<i32>, coords: &Coordinates){
    //
    // create a block with a unique name...
    //
    if aperture_id == None { panic!("tried to place an aperture before selecting it") }
    let some_aperture_id = aperture_id.unwrap();

    
}