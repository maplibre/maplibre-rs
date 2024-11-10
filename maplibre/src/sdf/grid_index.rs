//! Translated from https://github.com/maplibre/maplibre-native/blob/4add9ea/src/mbgl/util/grid_index.hpp

use std::{collections::HashSet, f64};

use crate::{
    euclid::{Box2D, Point2D},
    sdf::ScreenSpace,
};

#[derive(Default, Clone, Copy, Debug)]
pub struct Circle<T> {
    pub center: Point2D<T, ScreenSpace>,
    pub radius: T,
}

impl<T> Circle<T> {
    pub fn new(center: Point2D<T, ScreenSpace>, radius: T) -> Circle<T> {
        Self { center, radius }
    }
}

impl<T: PartialEq> PartialEq for Circle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.center == other.center && self.radius == other.radius
    }
}

impl<T: PartialEq> Eq for Circle<T> {}

pub struct GridIndex<T: Clone> {
    width: f64,
    height: f64,
    x_cell_count: usize,
    y_cell_count: usize,
    x_scale: f64,
    y_scale: f64,
    estimated_elements_per_cell: usize,
    box_elements: Vec<(T, Box2D<f64, ScreenSpace>)>,
    circle_elements: Vec<(T, Circle<f64>)>,
    box_cells: Vec<Vec<u32>>,
    circle_cells: Vec<Vec<u32>>,
}

impl<T: Clone> GridIndex<T> {
    pub fn new(width: f64, height: f64, cell_size: u32) -> Self {
        let x_cell_count = (width / cell_size as f64).ceil() as usize;
        let y_cell_count = (height / cell_size as f64).ceil() as usize;

        assert!(width > 0.0);
        assert!(height > 0.0);
        Self {
            width,
            height,
            x_cell_count,
            y_cell_count,
            x_scale: x_cell_count as f64 / width,
            y_scale: y_cell_count as f64 / height,
            estimated_elements_per_cell: 0,
            box_elements: vec![],
            circle_elements: vec![],
            box_cells: vec![vec![]; x_cell_count * y_cell_count],
            circle_cells: vec![vec![]; x_cell_count * y_cell_count],
        }
    }

    /// Set the expected number of elements per cell to avoid small re-allocations for populated cells
    pub fn reserve(&mut self, value: usize) {
        self.estimated_elements_per_cell = value;
    }

    pub fn insert(&mut self, t: T, bbox: Box2D<f64, ScreenSpace>) {
        assert!(self.box_elements.len() < u32::MAX as usize);
        let uid = self.box_elements.len() as u32;

        let cx1 = self.convert_to_x_cell_coord(bbox.min.x);
        let cy1 = self.convert_to_y_cell_coord(bbox.min.y);
        let cx2 = self.convert_to_x_cell_coord(bbox.max.x);
        let cy2 = self.convert_to_y_cell_coord(bbox.max.y);

        for x in cx1..=cx2 {
            for y in cy1..=cy2 {
                let cell = &mut self.box_cells[self.x_cell_count * y + x];
                if self.estimated_elements_per_cell > 0 && cell.is_empty() {
                    cell.reserve(self.estimated_elements_per_cell);
                }
                cell.push(uid);
            }
        }

        self.box_elements.push((t, bbox));
    }

    pub fn insert_circle(&mut self, t: T, circle: Circle<f64>) {
        assert!(self.circle_elements.len() < u32::MAX as usize);
        let uid = self.circle_elements.len() as u32;

        let cx1 = self.convert_to_x_cell_coord(circle.center.x - circle.radius);
        let cy1 = self.convert_to_y_cell_coord(circle.center.y - circle.radius);
        let cx2 = self.convert_to_x_cell_coord(circle.center.x + circle.radius);
        let cy2 = self.convert_to_y_cell_coord(circle.center.y + circle.radius);

        for x in cx1..=cx2 {
            for y in cy1..=cy2 {
                let cell = &mut self.circle_cells[self.x_cell_count * y + x];
                if self.estimated_elements_per_cell > 0 && cell.is_empty() {
                    cell.reserve(self.estimated_elements_per_cell);
                }
                cell.push(uid);
            }
        }

        self.circle_elements.push((t, circle));
    }

    pub fn query(&self, query_box: &Box2D<f64, ScreenSpace>) -> Vec<T> {
        let mut result = Vec::new();
        self.query_internal(query_box, |t, bbox| -> bool {
            result.push(t);
            return false;
        });
        return result;
    }

    pub fn query_with_boxes(
        &self,
        query_box: &Box2D<f64, ScreenSpace>,
    ) -> Vec<(T, Box2D<f64, ScreenSpace>)> {
        let mut result = Vec::new();
        self.query_internal(query_box, |t, bbox| -> bool {
            result.push((t, bbox));
            return false;
        });
        return result;
    }

    pub fn hit_test<F>(&self, query_box: &Box2D<f64, ScreenSpace>, predicate: Option<F>) -> bool
    where
        F: Fn(&T) -> bool,
    {
        let mut hit = false;
        self.query_internal(query_box, |t, _| -> bool {
            if let Some(predicate) = &predicate {
                if predicate(&t) {
                    hit = true;
                    return true;
                } else {
                    return false;
                }
            } else {
                hit = true;
                return true;
            }
        });
        return hit;
    }

    pub fn hit_test_circle<F>(&self, circle: &Circle<f64>, predicate: Option<F>) -> bool
    where
        F: Fn(&T) -> bool,
    {
        let mut hit = false;
        self.query_internal_circles(circle, |t, _| -> bool {
            if let Some(predicate) = &predicate {
                if predicate(&t) {
                    hit = true;
                    return true;
                } else {
                    return false;
                }
            } else {
                hit = true;
                return true;
            }
        });
        return hit;
    }

    pub fn empty(&self) -> bool {
        return self.box_elements.is_empty() && self.circle_elements.is_empty();
    }
}

impl<T: Clone> GridIndex<T> {
    fn no_intersection(&self, query_box: &Box2D<f64, ScreenSpace>) -> bool {
        return query_box.max.x < 0.0
            || query_box.min.x >= self.width
            || query_box.max.y < 0.0
            || query_box.min.y >= self.height;
    }

    fn complete_intersection(&self, query_box: &Box2D<f64, ScreenSpace>) -> bool {
        return query_box.min.x <= 0.0
            && query_box.min.y <= 0.0
            && self.width <= query_box.max.x
            && self.height <= query_box.max.y;
    }

    fn convert_to_box(circle: &Circle<f64>) -> Box2D<f64, ScreenSpace> {
        return Box2D::new(
            Point2D::new(
                circle.center.x - circle.radius,
                circle.center.y - circle.radius,
            ),
            Point2D::new(
                circle.center.x + circle.radius,
                circle.center.y + circle.radius,
            ),
        );
    }

    fn query_internal<F>(&self, query_bbox: &Box2D<f64, ScreenSpace>, mut result_fn: F)
    where
        F: FnMut(T, Box2D<f64, ScreenSpace>) -> bool,
    {
        let mut seen_boxes = HashSet::new();
        let mut seen_circles = HashSet::new();

        if self.no_intersection(query_bbox) {
            return;
        } else if self.complete_intersection(query_bbox) {
            for element in &self.box_elements {
                if result_fn(element.0.clone(), element.1) {
                    return;
                }
            }
            for element in &self.circle_elements {
                if result_fn(element.0.clone(), Self::convert_to_box(&element.1)) {
                    return;
                }
            }
            return;
        }

        let cx1 = self.convert_to_x_cell_coord(query_bbox.min.x);
        let cy1 = self.convert_to_y_cell_coord(query_bbox.min.y);
        let cx2 = self.convert_to_x_cell_coord(query_bbox.max.x);
        let cy2 = self.convert_to_y_cell_coord(query_bbox.max.y);

        let mut cell_index;
        for x in cx1..=cx2 {
            for y in cy1..=cy2 {
                cell_index = self.x_cell_count * y + x;
                // Look up other boxes
                for uid in &self.box_cells[cell_index] {
                    if !seen_boxes.contains(&uid) {
                        seen_boxes.insert(uid);

                        let pair = &self.box_elements[*uid as usize];
                        let bbox = pair.1;
                        if Self::boxes_collide(query_bbox, &bbox) {
                            if result_fn(pair.0.clone(), bbox) {
                                return;
                            }
                        }
                    }
                }

                // Look up circles
                for uid in &self.circle_cells[cell_index] {
                    if !seen_circles.contains(&uid) {
                        seen_circles.insert(uid);

                        let pair = &self.circle_elements[*uid as usize];
                        let bcircle = &pair.1;
                        if Self::circle_and_box_collide(&bcircle, query_bbox) {
                            if result_fn(pair.0.clone(), Self::convert_to_box(&bcircle)) {
                                return;
                            }
                        }
                    }
                }
            }
        }
    }

    fn query_internal_circles<F>(&self, query_bcircle: &Circle<f64>, mut result_fn: F)
    where
        F: FnMut(T, Box2D<f64, ScreenSpace>) -> bool,
    {
        let mut seen_boxes = HashSet::new();
        let mut seen_circles = HashSet::new();

        let query_bbox = Self::convert_to_box(query_bcircle);
        if self.no_intersection(&query_bbox) {
            return;
        } else if self.complete_intersection(&query_bbox) {
            for element in &self.box_elements {
                if result_fn(element.0.clone(), element.1) {
                    return;
                }
            }
            for element in &self.circle_elements {
                if result_fn(element.0.clone(), Self::convert_to_box(&element.1)) {
                    return;
                }
            }
        }

        let cx1 = self.convert_to_x_cell_coord(query_bcircle.center.x - query_bcircle.radius);
        let cy1 = self.convert_to_y_cell_coord(query_bcircle.center.y - query_bcircle.radius);
        let cx2 = self.convert_to_x_cell_coord(query_bcircle.center.x + query_bcircle.radius);
        let cy2 = self.convert_to_y_cell_coord(query_bcircle.center.y + query_bcircle.radius);

        let mut cell_index;
        for x in cx1..=cx2 {
            for y in cy1..=cy2 {
                cell_index = self.x_cell_count * y + x;
                // Look up boxes
                for uid in &self.box_cells[cell_index] {
                    if !seen_boxes.contains(&uid) {
                        seen_boxes.insert(uid);

                        let pair = &self.box_elements[*uid as usize];
                        let bbox = pair.1;
                        if Self::circle_and_box_collide(query_bcircle, &bbox) {
                            if result_fn(pair.0.clone(), bbox) {
                                return;
                            }
                        }
                    }
                }

                // Look up other circles
                for uid in &self.circle_cells[cell_index] {
                    if !seen_circles.contains(&uid) {
                        seen_circles.insert(uid);

                        let pair = &self.circle_elements[*uid as usize];
                        let bcircle = &pair.1;
                        if Self::circles_collide(query_bcircle, &bcircle) {
                            if result_fn(pair.0.clone(), Self::convert_to_box(&bcircle)) {
                                return;
                            }
                        }
                    }
                }
            }
        }
    }

    fn convert_to_x_cell_coord(&self, x: f64) -> usize {
        return f64::max(
            0.0,
            f64::min((self.x_cell_count - 1) as f64, f64::floor(x * self.x_scale)),
        ) as usize;
    }

    fn convert_to_y_cell_coord(&self, y: f64) -> usize {
        return f64::max(
            0.0,
            f64::min((self.y_cell_count - 1) as f64, f64::floor(y * self.y_scale)),
        ) as usize;
    }

    fn boxes_collide(first: &Box2D<f64, ScreenSpace>, second: &Box2D<f64, ScreenSpace>) -> bool {
        return first.min.x <= second.max.x
            && first.min.y <= second.max.y
            && first.max.x >= second.min.x
            && first.max.y >= second.min.y;
    }

    fn circles_collide(first: &Circle<f64>, second: &Circle<f64>) -> bool {
        let dx = second.center.x - first.center.x;
        let dy = second.center.y - first.center.y;
        let both_radii = first.radius + second.radius;
        return (both_radii * both_radii) > (dx * dx + dy * dy);
    }

    fn circle_and_box_collide(circle: &Circle<f64>, box_: &Box2D<f64, ScreenSpace>) -> bool {
        let half_rect_width = (box_.max.x - box_.min.x) / 2.0;
        let dist_x = (circle.center.x - (box_.min.x + half_rect_width)).abs();
        if dist_x > (half_rect_width + circle.radius) {
            return false;
        }

        let half_rect_height = (box_.max.y - box_.min.y) / 2.0;
        let dist_y = (circle.center.y - (box_.min.y + half_rect_height)).abs();
        if dist_y > (half_rect_height + circle.radius) {
            return false;
        }

        if dist_x <= half_rect_width || dist_y <= half_rect_height {
            return true;
        }

        let dx = dist_x - half_rect_width;
        let dy = dist_y - half_rect_height;
        return (dx * dx + dy * dy) <= (circle.radius * circle.radius);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexes_features() {
        let mut grid = GridIndex::<i16>::new(100.0, 100.0, 10);
        grid.insert(
            0,
            Box2D::new(Point2D::new(4.0, 10.0), Point2D::new(6.0, 30.0)),
        );
        grid.insert(
            1,
            Box2D::new(Point2D::new(4.0, 10.0), Point2D::new(30.0, 12.0)),
        );
        grid.insert(
            2,
            Box2D::new(Point2D::new(-10.0, 30.0), Point2D::new(5.0, 35.0)),
        );

        assert_eq!(
            grid.query(&Box2D::new(
                Point2D::new(4.0, 10.0),
                Point2D::new(5.0, 11.0)
            )),
            vec![0i16, 1]
        );
        assert_eq!(
            grid.query(&Box2D::new(
                Point2D::new(24.0, 10.0),
                Point2D::new(25.0, 11.0)
            )),
            vec![1i16]
        );
        let vec1: Vec<i16> = vec![];
        assert_eq!(
            grid.query(&Box2D::new(
                Point2D::new(40.0, 40.0),
                Point2D::new(100.0, 100.0)
            )),
            vec1
        );
        assert_eq!(
            grid.query(&Box2D::new(
                Point2D::new(-6.0, 0.0),
                Point2D::new(3.0, 100.0)
            )),
            vec![2i16]
        );
        assert_eq!(
            grid.query(&Box2D::new(
                Point2D::new(-1000.0, -1000.0),
                Point2D::new(1000.0, 1000.0)
            )),
            vec![0i16, 1, 2]
        );
    }
    #[test]
    fn duplicate_keys() {
        let mut grid = GridIndex::<i16>::new(100.0, 100.0, 10);
        const KEY: i16 = 123;
        grid.insert(
            KEY,
            Box2D::new(Point2D::new(3.0, 4.0), Point2D::new(4.0, 4.0)),
        );
        grid.insert(
            KEY,
            Box2D::new(Point2D::new(13.0, 13.0), Point2D::new(14.0, 14.0)),
        );
        grid.insert(
            KEY,
            Box2D::new(Point2D::new(23.0, 23.0), Point2D::new(24.0, 24.0)),
        );

        assert_eq!(
            grid.query(&Box2D::new(
                Point2D::new(0.0, 0.0),
                Point2D::new(30.0, 30.0)
            )),
            vec![KEY, KEY, KEY]
        );
    }

    fn i16_closure() {}

    #[test]
    fn circle_circle() {
        let mut grid = GridIndex::<i16>::new(100.0, 100.0, 10);
        grid.insert_circle(0, Circle::new(Point2D::new(50.0, 50.0), 10.0));
        grid.insert_circle(1, Circle::new(Point2D::new(60.0, 60.0), 15.0));
        grid.insert_circle(2, Circle::new(Point2D::new(-10.0, 110.0), 20.0));

        assert!(grid.hit_test_circle::<Box<dyn Fn(&i16) -> bool>>(
            &Circle::new(Point2D::new(55.0, 55.0), 2.0),
            None
        ));
        assert!(!grid.hit_test_circle::<Box<dyn Fn(&i16) -> bool>>(
            &Circle::new(Point2D::new(10.0, 10.0), 10.0),
            None
        ));
        assert!(grid.hit_test_circle::<Box<dyn Fn(&i16) -> bool>>(
            &Circle::new(Point2D::new(0.0, 100.0), 10.0),
            None
        ));
        assert!(grid.hit_test_circle::<Box<dyn Fn(&i16) -> bool>>(
            &Circle::new(Point2D::new(80.0, 60.0), 10.0),
            None
        ));
    }

    #[test]
    fn circle_box() {
        let mut grid = GridIndex::<i16>::new(100.0, 100.0, 10);
        grid.insert_circle(0, Circle::new(Point2D::new(50.0, 50.0), 10.0));
        grid.insert_circle(1, Circle::new(Point2D::new(60.0, 60.0), 15.0));
        grid.insert_circle(2, Circle::new(Point2D::new(-10.0, 110.0), 20.0));

        assert_eq!(
            grid.query(&Box2D::new(
                Point2D::new(45.0, 45.0),
                Point2D::new(55.0, 55.0)
            )),
            vec![0, 1]
        );
        let vec1: Vec<i16> = vec![];
        assert_eq!(
            grid.query(&Box2D::new(
                Point2D::new(0.0, 0.0),
                Point2D::new(30.0, 30.0)
            )),
            vec1
        );
        assert_eq!(
            grid.query(&Box2D::new(
                Point2D::new(0.0, 80.0),
                Point2D::new(20.0, 100.0)
            )),
            vec![2]
        );
    }

    #[test]
    fn indexes_features_overflow() {
        let mut grid = GridIndex::<i16>::new(5000.0, 5000.0, 25);
        grid.insert(
            0,
            Box2D::new(Point2D::new(4500.0, 4500.0), Point2D::new(4900.0, 4900.0)),
        );
        assert_eq!(
            grid.query(&Box2D::new(
                Point2D::new(4000.0, 4000.0),
                Point2D::new(5000.0, 5000.0)
            )),
            vec![0]
        );
    }
}
