use cgmath::{Matrix, Matrix3, MetricSpace, SquareMatrix, Transform, Vector2, Zero};
use std::collections::HashMap;
use std::fmt::Debug;
use log::info;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ButtonSource, ElementState, FingerId, PointerKind, PointerSource, WindowEvent};

#[derive(Debug, Clone)]
pub struct Fingers {
    fingers: HashMap<FingerId, Finger>,
    view_matrix_before_transform: Matrix3<f64>,
}

#[derive(Debug, Clone)]
pub struct Finger {
    start_position: Vector2<f64>,
    current_position: Vector2<f64>,
}

impl Finger {
    fn new(position: &Vector2<f64>) -> Self {
        Self {
            start_position: position.clone(),
            current_position: position.clone(),
        }
    }
    
    fn update_position(&mut self, position: &Vector2<f64>) {
        self.current_position = position.clone();
    }
    
    fn reset(&mut self) {
        self.start_position = self.current_position.clone();
    }
}

impl Fingers {
    pub fn new() -> Self {
        Self {
            fingers: Default::default(),
            view_matrix_before_transform: Matrix3::identity(),
        }
    }
    
    fn reset(&mut self, view_matrix: Matrix3<f64>) {
        self.view_matrix_before_transform = view_matrix;
        for (_, finger) in &mut self.fingers {
            finger.reset()
        }
    }
    
    pub(crate) fn handle_event(
        &mut self, 
        event: &WindowEvent, 
        window_size: PhysicalSize<f64>,
        view_matrix: Matrix3<f64>,
    ) -> Matrix3<f64> {
        // Mapping the screen position so that the center is at 0,0:
        let screen_to_vector = |screen: &PhysicalPosition<f64>| {
            Vector2::new(
                screen.x - window_size.width/2.0,
                screen.y - window_size.height/2.0,
            )
        };
        match event {
            WindowEvent::PointerButton {
                button: ButtonSource::Touch { finger_id, .. },
                position,
                state: ElementState::Pressed,
                ..
            } | WindowEvent::PointerMoved {
                source: PointerSource::Touch { finger_id, .. },
                position,
                ..
            } => {
                let position = screen_to_vector(position);
                if let Some(finger) = self.fingers.get_mut(finger_id) {
                    finger.update_position(&position);
                } else {
                    info!("{:?}", view_matrix - self.view_matrix());
                    self.reset(view_matrix);
                    self.fingers.insert(finger_id.clone(), Finger::new(&position));
                }
            }
            WindowEvent::PointerButton {
                button: ButtonSource::Touch { finger_id, .. },
                state: ElementState::Released,
                ..
            } | WindowEvent::PointerLeft { 
                kind: PointerKind::Touch(finger_id),
                ..
            } => {
                info!("{:?}", view_matrix - self.view_matrix());
                self.reset(view_matrix);
                self.fingers.remove(finger_id);
            }
            _ => {}
        }
        self.view_matrix()
    }
    
    fn view_matrix(&self) -> Matrix3<f64> {
        self.pan_and_zoom_matrix() * self.view_matrix_before_transform
    }
    fn view_matrix_inverse(&self) -> Matrix3<f64> {
        self.view_matrix().invert().unwrap_or(Matrix3::identity())
    }
    fn pre_matrix_inverse(&self) -> Matrix3<f64> {
        self.view_matrix_before_transform.invert().unwrap_or(Matrix3::identity())
    }
    
    fn world_center_of_pinch_before_transform(&self) -> Option<Vector2<f64>> {
        let inverse = self.view_matrix_before_transform.invert()?;
        Some((inverse * self.screen_center_of_pinch_before_transform()?.extend(1.0)).truncate())
    }
    fn world_center_of_pinch_after_transform(&self) -> Option<Vector2<f64>> {
        let inverse = self.view_matrix().invert()?;
        Some((inverse * self.screen_center_of_pinch_after_transform()?.extend(1.0)).truncate())
    }
    fn screen_center_of_pinch_before_transform(&self) -> Option<Vector2<f64>> {
        let start_points: Vec<_> = self.fingers.iter().map(|(_, finger)| { finger.start_position }).collect();
        Some(BoundingBox::from_points(start_points)?.center())
    }
    fn screen_center_of_pinch_after_transform(&self) -> Option<Vector2<f64>> {
        let current_points: Vec<_> = self.fingers.iter().map(|(_, finger)| { finger.current_position }).collect();
        Some(BoundingBox::from_points(current_points)?.center())
    }
    
    /// The matrix is representing a zoom followed by a pan
    fn pan_and_zoom_matrix(&self) -> Matrix3<f64> {
        let scale = self.pinch_size_increase().unwrap_or(1.0);
        let pan = self.pan();
        Matrix3::new(
            scale, 0.0, scale*pan.x,
            0.0, scale, scale*pan.y,
            0.0, 0.0, 1.0,
        ).transpose()
    }
    
    fn pinch_size_increase(&self) -> Option<f64> {
        let start_points: Vec<_> = self.fingers.iter().map(|(_, finger)| { finger.start_position }).collect();
        let current_points: Vec<_> = self.fingers.iter().map(|(_, finger)| { finger.current_position }).collect();
        let start_bb = BoundingBox::from_points(start_points)?;
        let current_bb = BoundingBox::from_points(current_points)?;
        if start_bb.size() < 0.01 {
            return None;
        }
        Some(current_bb.size() / start_bb.size())
    }
    
    fn pan(&self) -> Vector2<f64> {
        let start_points: Vec<_> = self.fingers.iter().map(|(_, finger)| { finger.start_position }).collect();
        let current_points: Vec<_> = self.fingers.iter().map(|(_, finger)| { finger.current_position }).collect();
        let start_bb = BoundingBox::from_points(start_points).unwrap_or_default();
        let current_bb = BoundingBox::from_points(current_points).unwrap_or_default();
        current_bb.center() - start_bb.center()
    }
}

struct BoundingBox {
    minimum: Vector2<f64>,
    maximum: Vector2<f64>,
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self {
            minimum: Vector2::zero(),
            maximum: Vector2::zero(),
        }
    }
}

impl BoundingBox {
    fn from_points(points: Vec<Vector2<f64>>) -> Option<Self> {
        let first = points.first()?;
        let mut minimum = first.clone();
        let mut maximum = first.clone();
        for point in points {
            if point.x < minimum.x { minimum.x = point.x }
            if point.y < minimum.y { minimum.y = point.y }
            if point.x > maximum.x { maximum.x = point.x }
            if point.y > maximum.y { maximum.y = point.y }
        }
        Some(BoundingBox {
            minimum, maximum
        })
    }
    
    fn size(&self) -> f64 {
        self.minimum.distance(self.maximum)
    }
    
    fn center(&self) -> Vector2<f64> {
        (self.minimum + self.maximum) / 2.0
    }
}

