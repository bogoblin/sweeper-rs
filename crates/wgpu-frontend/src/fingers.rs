use cgmath::{Matrix, Matrix3, MetricSpace, Vector2, Zero};
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ButtonSource, ElementState, FingerId, PointerKind, PointerSource, WindowEvent};

#[derive(Debug, Clone)]
pub struct Fingers {
    fingers: HashMap<FingerId, Finger>,
    view_matrix_before_transform: Matrix3<f64>,
    released: VecDeque<Finger>,
}

#[derive(Debug, Clone)]
pub struct Finger {
    touched_position: Vector2<f64>,
    position_at_begin_transform: Vector2<f64>,
    current_position: Vector2<f64>,
}

impl Finger {
    fn new(position: &Vector2<f64>) -> Self {
        Self {
            touched_position: position.clone(),
            position_at_begin_transform: position.clone(),
            current_position: position.clone(),
        }
    }
    
    fn update_position(&mut self, position: &Vector2<f64>) {
        self.current_position = position.clone();
    }
    
    fn reset(&mut self) {
        self.position_at_begin_transform = self.current_position.clone();
    }

    pub fn distance_moved(&self) -> f64 {
        self.touched_position.distance(self.current_position)
    }

    pub fn screen_position(&self) -> Vector2<f64> {
        self.current_position
    }
}

impl Fingers {
    pub fn new(view_matrix: Matrix3<f64>) -> Self {
        Self {
            fingers: Default::default(),
            view_matrix_before_transform: view_matrix,
            released: Default::default(),
        }
    }
    
    fn reset(&mut self, view_matrix: Matrix3<f64>) {
        self.view_matrix_before_transform = view_matrix;
        for (_, finger) in &mut self.fingers {
            finger.reset()
        }
    }

    pub fn next_released_finger(&mut self) -> Option<Finger> {
        self.released.pop_front()
    }
    
    pub(crate) fn handle_event(
        &mut self, 
        event: &WindowEvent, 
        window_size: PhysicalSize<f64>,
        view_matrix: Matrix3<f64>,
    ) -> Option<Matrix3<f64>> {
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
                    self.reset(view_matrix);
                    self.fingers.insert(finger_id.clone(), Finger::new(&position));
                }
                Some(self.view_matrix())
            }
            WindowEvent::PointerButton {
                button: ButtonSource::Touch { finger_id, .. },
                state: ElementState::Released,
                ..
            } | WindowEvent::PointerLeft { 
                kind: PointerKind::Touch(finger_id),
                ..
            } => {
                self.reset(view_matrix);
                if let Some(removed) = self.fingers.remove(finger_id) {
                    self.released.push_back(removed);
                } else {
                    
                }
                Some(self.view_matrix())
            }
            _ => None
        }
    }
    
    fn view_matrix(&self) -> Matrix3<f64> {
        self.pan_and_zoom_matrix() * self.view_matrix_before_transform
    }
    
    /// The matrix is representing a zoom followed by a pan
    fn pan_and_zoom_matrix(&self) -> Matrix3<f64> {
        let scale = self.pinch_size_increase().unwrap_or(1.0);
        
        let start_points: Vec<_> = self.fingers.iter().map(|(_, finger)| { finger.position_at_begin_transform }).collect();
        let start_bb = BoundingBox::from_points(start_points).unwrap_or_default();
        let start_center = start_bb.center();
        // Start with the matrix that translates the start center to (0,0,1)
        let mut matrix = Matrix3::new(
            1.0, 0.0, -start_center.x,
            0.0, 1.0, -start_center.y,
            0.0, 0.0, 1.0,
        ).transpose();
        
        // Then scale it:
        matrix = Matrix3::new(
            scale, 0.0, 0.0,
            0.0, scale, 0.0,
            0.0, 0.0, 1.0,
        ).transpose() * matrix;

        let current_points: Vec<_> = self.fingers.iter().map(|(_, finger)| { finger.current_position }).collect();
        let current_bb = BoundingBox::from_points(current_points).unwrap_or_default();
        let current_center = current_bb.center();
        // Then translate so that the center is where the new center is:
        matrix = Matrix3::new(
            1.0, 0.0, current_center.x,
            0.0, 1.0, current_center.y,
            0.0, 0.0, 1.0,
        ).transpose() * matrix;
        
        matrix
    }
    
    fn pinch_size_increase(&self) -> Option<f64> {
        let start_points: Vec<_> = self.fingers.iter().map(|(_, finger)| { finger.position_at_begin_transform }).collect();
        let current_points: Vec<_> = self.fingers.iter().map(|(_, finger)| { finger.current_position }).collect();
        let start_bb = BoundingBox::from_points(start_points)?;
        let current_bb = BoundingBox::from_points(current_points)?;
        if start_bb.size() < 0.01 {
            return None;
        }
        Some(current_bb.size() / start_bb.size())
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

