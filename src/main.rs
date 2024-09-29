use std::fs::File;
use std::io::BufReader;

use gerber_parser::gerber_doc::GerberDoc;
use gerber_parser::parser::parse_gerber;
use gerber_types::{Aperture, Command, Coordinates, DCode, GCode, InterpolationMode, Operation, Unit};
use gerber_types::{CoordinateOffset, FunctionCode};
use dxf;
use dxf::entities::{Entity, Insert};
use dxf::Point;

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

    let mut current_path: Vec<dxf::Point> = Vec::new();

    let mut drawing = dxf::Drawing::new();
    drawing.header.default_drawing_units = units;
    
    let mut current_aperture: Option<i32> = None;
    let mut region_mode = false;
    
    for (id, aperture) in &gerber_doc.apertures{
        let mut block = dxf::Block {
            name: aperture_id_as_string(*id),
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
            Aperture::Rectangle(rectangle) => {
                let half_x = rectangle.x/2.0;
                let half_y = rectangle.y/2.0;
                let points: [dxf::Point; 4] = [
                    dxf::Point::new( half_x,  half_y, 0.0),
                    dxf::Point::new(-half_x,  half_y, 0.0),
                    dxf::Point::new(-half_x, -half_y, 0.0),
                    dxf::Point::new( half_x, -half_y, 0.0),
                ];
                block.entities.push(dxf::entities::Entity {
                    common: Default::default(),
                    specific: dxf::entities::EntityType::Line(
                        dxf::entities::Line::new(
                            points[0].clone(), points[1].clone()
                        )
                    )
                });
                block.entities.push(dxf::entities::Entity {
                    common: Default::default(),
                    specific: dxf::entities::EntityType::Line(
                        dxf::entities::Line::new(
                            points[1].clone(), points[2].clone()
                        )
                    )
                });
                block.entities.push(dxf::entities::Entity {
                    common: Default::default(),
                    specific: dxf::entities::EntityType::Line(
                        dxf::entities::Line::new(
                            points[2].clone(), points[3].clone()
                        )
                    )
                });
                block.entities.push(dxf::entities::Entity {
                    common: Default::default(),
                    specific: dxf::entities::EntityType::Line(
                        dxf::entities::Line::new(
                            points[3].clone(), points[0].clone()
                        )
                    )
                });
            }
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
                                    Operation::Interpolate(coords, _offset) => {
                                        add_to_path(&mut current_path, coords);
                                    }
                                    Operation::Move(coords) => {
                                        add_interpolation(&mut drawing, &gerber_doc, &mut current_path, current_aperture);
                                        add_to_path(&mut current_path, coords);
                                    }
                                    Operation::Flash(coords) => {
                                        add_interpolation(&mut drawing, &gerber_doc, &mut current_path, current_aperture);
                                        flash_aperture_at_coords(&mut drawing, current_aperture, &coord_to_point(coords).expect("flash at bad coords!"));
                                    }
                                }
                            }
                            DCode::SelectAperture(aperture_id) => {
                                add_interpolation(&mut drawing, &gerber_doc, &mut current_path, current_aperture);
                                current_aperture = Some(*aperture_id);
                            }
                        }
                    }
                    FunctionCode::GCode(gc) => {
                        add_interpolation(&mut drawing, &gerber_doc, &mut current_path, current_aperture);
                        match gc{
                            GCode::InterpolationMode(_) => {}
                            GCode::RegionMode(_is_begin) => {
                                current_aperture = None
                                // TODO: Regions?
                            }
                            GCode::QuadrantMode(_) => {}
                            GCode::Comment(_) => {}
                        }
                    }
                    FunctionCode::MCode(_) => {add_interpolation(&mut drawing, &gerber_doc, &mut current_path, current_aperture);}
                }
            }
            Command::ExtendedCode(ec) => {add_interpolation(&mut drawing, &gerber_doc, &mut current_path, current_aperture);}
        }
    }

    drawing.save_file("test_files/test_dxf.dxf").expect("failed to save");
}

fn add_interpolation(
    drawing: &mut dxf::Drawing,
    gerber_doc: &GerberDoc,
    coord_list: &mut Vec<Point>,
    aperture_id: Option<i32>
) {
    if aperture_id != None{
        match coord_list.len(){
            0usize => {/* Do nothing */}
            1usize => {
                flash_aperture_at_coords(drawing, aperture_id, &coord_list[0])
            }
            length => {
                println!("add interpolation of length {length}");
                match get_aperture(gerber_doc, aperture_id){
                    Aperture::Circle(circle) => {
                        add_circle_interpolation(drawing, coord_list, circle.diameter)
                    }
                    _ => {
                        unimplemented!("Currently only circle interpolations are supported")
                    }
                }
            }
        }
    } else { // assume no selected aperture means region mode
        let mut vertex_vec: Vec<dxf::LwPolylineVertex> = Vec::new();
        for point in coord_list.iter(){
            vertex_vec.push(dxf::LwPolylineVertex { 
                x: point.x,
                y: point.y,
                ..Default::default()
            });
        }
        drawing.add_entity(dxf::entities::Entity {
            common: Default::default(),
            specific: dxf::entities::EntityType::LwPolyline(dxf::entities::LwPolyline{
                vertices: vertex_vec,
                ..Default::default()
            }),
        });
    }

    
    coord_list.clear();
}

fn flash_aperture_at_coords(drawing: &mut dxf::Drawing, aperture_id: Option<i32>, coords: &Point){
    //
    // create a block with a unique name...
    //
    if aperture_id == None { panic!("tried to place an aperture before selecting it") }
    let some_aperture_id = aperture_id.unwrap();
    let insert: Insert = Insert{
        name: aperture_id_as_string(some_aperture_id),
        location: coords.clone(),
        ..Default::default()
    };
    drawing.add_entity(dxf::entities::Entity {
        common: Default::default(),
        specific: dxf::entities::EntityType::Insert(insert),
    });
}

fn get_aperture(gerber_doc: &GerberDoc, aperture_id: Option<i32>) -> &Aperture {
    if aperture_id == None { panic!("tried to get an aperture before selecting it") }
    let some_aperture_id = aperture_id.unwrap();
    gerber_doc.apertures.get(&some_aperture_id).expect("tried to get nonexistent aperture")
}

fn aperture_id_as_string(aperture_id: i32) -> String {
    format!("aperture_id_{}", aperture_id)
}

fn coord_to_point(coords: &Coordinates) -> Option<dxf::Point> {
    Some(dxf::Point::new(
        coords.x?.into(),
        coords.y?.into(),
        0.0
    ))
}

#[derive(Copy, Clone)]
struct Vector{
    x: f64,
    y: f64,
}

impl Vector{
    /// translates the vector that starts at `point_a` and ends at `point_b` to start at 0,0
    pub fn from_points(point_a: &dxf::Point, point_b: &dxf::Point) -> Vector {
        Vector{
            x: point_b.x - point_a.x,
            y: point_b.y - point_a.y,
        }
    }
    pub fn get_normalized(&self) -> Vector{
        self.with_magnitude(1.0)
    }
    
    pub fn reverse(&mut self){
        self.x = -self.x;
        self.y = -self.y;
    }
    
    pub fn get_reversed(&self) -> Vector{
        Vector{
            x: -self.x,
            y: -self.y,
        }
    }
    
    pub fn get_rotate_cw(&self) -> Vector{
        Vector {
            x: -self.y,
            y:  self.x
        }
    }

    pub fn get_rotate_ccw(&self) -> Vector{
        Vector {
            x:  self.y,
            y: -self.x
        }
    }
    
    pub fn with_magnitude(&self, magnitude: f64) -> Vector{
        let current_magnitude = self.get_magnitude();
        Vector {
            x: magnitude*self.x/current_magnitude,
            y: magnitude*self.y/current_magnitude
        }
    }
    
    pub fn get_angle_degrees(&self) -> f64 {
        let normalized_vec = self.with_magnitude(1.0);
        
        let mut degs = normalized_vec.x.acos();
        
        if (normalized_vec.y < 0.0) {
            degs = core::f64::consts::TAU-degs;
        }
        degs.to_degrees()
    }
    
    pub fn get_magnitude(&self) -> f64{
        ((self.x * self.x) + (self.y * self.y)).sqrt()
    }
    
    pub fn apply(&self, point: &dxf::Point) -> dxf::Point {
        dxf::Point::new(point.x+self.x, point.y+self.y, point.z)
    }
}


fn add_circle_interpolation(drawing: &mut dxf::Drawing, coord_list: &mut Vec<Point>, diameter: f64){
    let len: usize = coord_list.len();
    
    
    for i in 0..len-1{
        let point_a = &coord_list[i];
        let point_b = &coord_list[i + 1];
        let point_c_maybe = coord_list.get(i + 2);
        let vec = Vector::from_points(point_a, point_b);
        let cw_vec = vec.get_rotate_cw().with_magnitude(diameter/2.0);
        let cw_line = dxf::entities::Line::new(cw_vec.apply(point_a), cw_vec.apply(point_b));
        let ccw_vec = vec.get_rotate_ccw().with_magnitude(diameter/2.0);
        let ccw_line = dxf::entities::Line::new(ccw_vec.apply(point_a), ccw_vec.apply(point_b));
        drawing.add_entity(dxf::entities::Entity {
            common: Default::default(),
            specific: dxf::entities::EntityType::Line(cw_line),
        });
        drawing.add_entity(dxf::entities::Entity {
            common: Default::default(),
            specific: dxf::entities::EntityType::Line(ccw_line),
        });
        match point_c_maybe{
            Some(point_c) => {
                drawing.add_entity(dxf::entities::Entity {
                    common: Default::default(),
                    specific: dxf::entities::EntityType::Arc(
                        interpolation_arc(point_a, point_b, point_c, diameter/2.0)
                    ),
                });
            }
            None => {}
        }
    }



    let first_vec = Vector::from_points(
        &coord_list[0],
        &coord_list[1]
    );
    
    let radius_vec = first_vec.with_magnitude(diameter/2.0);
    
    let first_point = &coord_list[0];
    
    
    drawing.add_entity(dxf::entities::Entity{
        common: Default::default(),
        specific: dxf::entities::EntityType::Arc(
            arc_from_points_and_center(
                &radius_vec.get_rotate_cw().apply(first_point),
                &radius_vec.get_rotate_ccw().apply(first_point),
                first_point,
                diameter/2.0
            )
        )
    });

    let last_vec = Vector::from_points(
        &coord_list[len-1],
        &coord_list[len-2]
    );

    let radius_vec = last_vec.with_magnitude(diameter/2.0);

    let last_point = &coord_list[len-1];


    drawing.add_entity(dxf::entities::Entity{
        common: Default::default(),
        specific: dxf::entities::EntityType::Arc(
            arc_from_points_and_center(
                &radius_vec.get_rotate_cw().apply(last_point),
                &radius_vec.get_rotate_ccw().apply(last_point),
                last_point,
                diameter/2.0
            )
        )
    });
}

/// this is technically more than enough information to constrain an arc, 
/// ^ Actually that's still not true, there are 2 possibilities, but arc goes ccw
/// but this is the info that is readily available to the caller
fn arc_from_points_and_center(point_a: &dxf::Point,
                              point_b: &dxf::Point,
                              center: &dxf::Point,
                              radius: f64
) -> dxf::entities::Arc {
    let vec_a = Vector::from_points(&center, &point_a);
    let vec_b = Vector::from_points(&center, &point_b);
    
    
    // TODO: Arc always goes counter clockwise, very cringe.
    dxf::entities::Arc {
        center: center.clone(),
        radius,
        start_angle: vec_a.get_angle_degrees(),
        end_angle: vec_b.get_angle_degrees(),
        ..Default::default()
    }
}


fn interpolation_arc(point_a: &dxf::Point, point_b: &dxf::Point, point_c: &dxf::Point, radius: f64
) -> dxf::entities::Arc{
    let a_rad_vec = Vector::from_points(point_b, point_a).with_magnitude(radius);
    let c_rad_vec = Vector::from_points(point_b, point_c).with_magnitude(radius);
    
    let a_to_c_ccw_deg_delta = (a_rad_vec.get_angle_degrees() - c_rad_vec.get_angle_degrees()) % 360.0;
            
    if a_to_c_ccw_deg_delta > 180.0 {
        let point_a = a_rad_vec.get_rotate_ccw().apply(point_b);
        let point_c = c_rad_vec.get_rotate_cw().apply(point_b);
        arc_from_points_and_center(&point_c, &point_a, point_b, radius)
    } else { // 'above'
        let point_a = a_rad_vec.get_rotate_cw().apply(point_b);
        let point_c = c_rad_vec.get_rotate_ccw().apply(point_b);
        arc_from_points_and_center(&point_a, &point_c, point_b, radius)
    }
}


fn add_to_path(path: &mut Vec<dxf::Point>, coord_to_add: &Coordinates){
    match coord_to_point(coord_to_add){
        None => {
            println!("bad path coord")
        }
        Some(point) => {
            path.push(point);
        }
    }
}









