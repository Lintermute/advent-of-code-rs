use core::{borrow::Borrow, fmt, hash::Hash, str::FromStr};

use std::collections::{hash_map, HashMap, HashSet};

use lazy_errors::{prelude::*, Result};

use crate::parser::{
    self,
    vec2::{IVec2, UVec2},
    Direction, Point, Rect, Vec2,
};

const MSG_INCONSISTENT: &str = "Grid state is inconsistent";

pub trait Position {
    fn position(&self) -> &Point;
}

pub trait RenderOn<A> {
    fn render_on(&self, area: &A) -> Result<Vec<Vec<char>>>;
}

#[derive(Debug, PartialEq, Hash, Eq)]
pub struct Entity<A, T> {
    area: A,
    data: T,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid<A = Point, T = char>
where
    A: Hash + Eq,
{
    bounds: Rect,
    grid:   UnboundedGrid<A, T>,
}

#[derive(Debug, Default, PartialEq, Eq)] // Implements `Clone` manually.
struct UnboundedGrid<A = Point, T = char>
where
    A: Hash + Eq,
{
    next_id: usize,
    area_to_id: HashMap<A, usize>,
    point_to_id: HashMap<Point, usize>, // TODO: Redundancy if `A == Point`
    id_to_entity: HashMap<usize, Entity<A, T>>,
}

impl<A, T> Entity<A, T> {
    pub fn area(&self) -> &A {
        &self.area
    }

    pub fn data(&self) -> &T {
        &self.data
    }
}

impl Grid<Point, ()> {
    pub fn from_points(
        bounds: Rect,
        points: impl IntoIterator<Item = Point>,
    ) -> Result<Self> {
        let tuples = points.into_iter().map(|p| (p, ()));
        Self::from(bounds, tuples)
    }
}

impl<A, T> Grid<A, T>
where
    A: Into<Vec<Point>> + Clone + Hash + Eq,
{
    pub fn from_str<'a, E, I>(
        input: &'a str,
        matcher: impl FnMut(&'a str) -> I,
    ) -> Result<Self>
    where
        A: TryFrom<Rect> + TryFrom<Point>,
        <A as TryFrom<Rect>>::Error: Into<Stashable>,
        <A as TryFrom<Point>>::Error: Into<Stashable>,
        T: FromStr<Err = E>,
        E: Into<Stashable>,
        I: Iterator<Item = (usize, usize)>,
    {
        Self::from_lines(input.lines(), matcher)
    }

    pub fn from_lines<'a, E, I>(
        lines: impl Iterator<Item = &'a str>,
        mut matcher: impl FnMut(&'a str) -> I,
    ) -> Result<Self>
    where
        A: TryFrom<Rect> + TryFrom<Point>,
        <A as TryFrom<Rect>>::Error: Into<Stashable>,
        <A as TryFrom<Point>>::Error: Into<Stashable>,
        T: FromStr<Err = E>,
        E: Into<Stashable>,
        I: Iterator<Item = (usize, usize)>,
    {
        let mut errs = ErrorStash::new(|| "Failed to parse grid");

        let mut grid = UnboundedGrid::<A, T>::new();

        let mut y_max = 0;
        let mut x_max = HashSet::<usize>::new();
        for (y, line) in lines.enumerate() {
            y_max = y;
            x_max.insert(line.len());

            for (x, dx) in matcher(line) {
                parser::parse_substr(y, x, dx, line)
                    .and_then(|(area, data)| grid.insert(area, data))
                    .or_stash(&mut errs);
            }
        }

        let x_max = match Vec::from_iter(x_max).as_slice() {
            &[] => 0,
            &[x_max] => x_max,
            others => {
                let errs = errs.push_and_convert(format!(
                    "Line lengths differ: {others:?}"
                ));
                return Err(errs.into());
            }
        };

        // Bail if there were any errors:
        errs.into_result()?;

        let bounds = Vec2::new(y_max + 1, x_max + 1);
        let bounds = Rect::new(Point::ZERO, bounds)?;

        // `bounds` is computed from actual data, so that check "cannot" fail:
        assert!(grid
            .iter()
            .all(|e| bounds.contains_all(e.area().clone())));

        Ok(Self { bounds, grid })
    }

    pub fn from(
        bounds: Rect,
        iter: impl IntoIterator<Item = (A, T)>,
    ) -> Result<Self> {
        let mut errs = ErrorStash::new(|| "Failed to create grid");

        let mut grid = Self {
            bounds,
            grid: UnboundedGrid::new(),
        };

        for (area, kind) in iter {
            grid.insert(area, kind)
                .or_stash(&mut errs);
        }

        errs.into_result().map(|()| grid)
    }

    pub fn try_from(
        bounds: Rect,
        iter: impl IntoIterator<Item = Result<(A, T)>>,
    ) -> Result<Self> {
        let mut errs = ErrorStash::new(|| "Failed to create grid");

        let mut grid = Self {
            bounds,
            grid: UnboundedGrid::new(),
        };

        for item in iter {
            let (area, kind) = item?;
            grid.insert(area, kind)
                .or_stash(&mut errs);
        }

        errs.into_result().map(|()| grid)
    }

    pub fn bounds(&self) -> &Rect {
        &self.bounds
    }

    pub fn get_at(&self, p: &Point) -> Option<&Entity<A, T>> {
        self.grid.get_at(p)
    }

    pub fn get_at_any<'a, P>(
        &'a self,
        points: impl IntoIterator<Item = P>,
    ) -> impl Iterator<Item = &'a Entity<A, T>>
    where
        P: Borrow<Point> + 'a,
    {
        self.grid.get_at_any(points)
    }

    pub fn get_at_exactly(&self, a: &A) -> Option<&Entity<A, T>> {
        self.grid.get_at_exactly(a)
    }

    pub fn get_data_at(&self, p: &Point) -> Option<&T> {
        self.grid.get_data_at(p)
    }

    pub fn find_all<'a, 'b>(
        &'a self,
        data: &'b T,
    ) -> impl Iterator<Item = &'a Entity<A, T>> + use<'a, 'b, A, T>
    where
        T: PartialEq,
    {
        self.grid.find_all(data)
    }

    pub fn find_exactly_one(&self, data: &T) -> Result<&Entity<A, T>>
    where
        T: PartialEq,
    {
        self.grid.find_exactly_one(data)
    }

    pub fn find_all_neighbors<'a, 'b>(
        &'a self,
        p: &'b Point,
    ) -> impl Iterator<Item = (&'a Entity<A, T>, Direction)> + use<'a, 'b, A, T>
    {
        self.grid.find_all_neighbors(p)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Entity<A, T>> {
        self.into_iter() // takes `&self` by ref
    }

    pub fn insert(&mut self, area: A, data: T) -> Result<()> {
        self.grid
            .insert_within(&self.bounds, area, data)
    }

    pub fn move_by_from(&mut self, v: IVec2, p: &Point) -> Result<()>
    where
        A: core::ops::Add<IVec2, Output = A>,
    {
        self.grid
            .move_within_by_from(&self.bounds, v, p)
    }

    pub fn move_by_from_any<'a>(
        &mut self,
        v: IVec2,
        points: impl IntoIterator<Item = &'a Point>,
    ) -> Result<()>
    where
        A: core::ops::Add<IVec2, Output = A>,
    {
        self.grid
            .move_within_by_from_any(&self.bounds, v, points)
    }

    pub fn extract(&mut self, e: &Entity<A, T>) -> Option<Entity<A, T>> {
        self.grid.extract(e)
    }

    pub fn extract_at(&mut self, p: &Point) -> Option<Entity<A, T>> {
        self.grid.extract_at(p)
    }

    pub fn extract_at_any<'a, I>(
        &mut self,
        points: I,
    ) -> impl Iterator<Item = Entity<A, T>> + use<'_, 'a, A, T, I>
    where
        I: IntoIterator<Item = &'a Point>,
    {
        self.grid.extract_at_any(points)
    }

    pub fn extract_exactly_one(&mut self, data: &T) -> Result<Entity<A, T>>
    where
        T: PartialEq,
    {
        self.grid.extract_exactly_one(data)
    }
}

impl<A, T> UnboundedGrid<A, T>
where
    A: Into<Vec<Point>> + Clone + Hash + Eq,
{
    pub fn new() -> Self {
        Self {
            next_id: 0,
            area_to_id: HashMap::new(),
            point_to_id: HashMap::new(),
            id_to_entity: HashMap::new(),
        }
    }

    pub fn get_at(&self, p: &Point) -> Option<&Entity<A, T>> {
        let id = self.get_at_impl(p)?;
        Some(self.get_by_id_or_panic(id))
    }

    pub fn get_at_any<'a, P>(
        &'a self,
        points: impl IntoIterator<Item = P>,
    ) -> impl Iterator<Item = &'a Entity<A, T>>
    where
        P: Borrow<Point> + 'a,
    {
        use itertools::Itertools;
        points
            .into_iter()
            .flat_map(|p| self.point_to_id.get(p.borrow()))
            .unique()
            .map(|&id| self.get_by_id_or_panic(id))
    }

    pub fn get_at_exactly(&self, a: &A) -> Option<&Entity<A, T>> {
        let id = self.get_at_exactly_impl(a)?;
        Some(self.get_by_id_or_panic(id))
    }

    pub fn get_data_at(&self, p: &Point) -> Option<&T> {
        self.get_at(p).map(|e| &e.data)
    }

    pub fn find_all<'a, 'b>(
        &'a self,
        data: &'b T,
    ) -> impl Iterator<Item = &'a Entity<A, T>> + use<'a, 'b, A, T>
    where
        T: PartialEq,
    {
        self.find_all_impl(data)
            .map(|id| self.get_by_id_or_panic(id))
    }

    pub fn find_exactly_one(&self, data: &T) -> Result<&Entity<A, T>>
    where
        T: PartialEq,
    {
        self.find_exactly_one_impl(data)
            .and_then(|id| self.get_by_id(id))
    }

    pub fn find_all_neighbors<'a, 'b>(
        &'a self,
        p: &'b Point,
    ) -> impl Iterator<Item = (&'a Entity<A, T>, Direction)> + use<'a, 'b, A, T>
    {
        Direction::ALL
            .iter()
            .map(|&d| {
                let p = *p + IVec2::from(d);
                (p, d)
            })
            .flat_map(|(p, d)| self.get_at(&p).map(|e| (e, d)))
    }

    pub fn iter(&self) -> impl Iterator<Item = &Entity<A, T>> {
        self.into_iter() // takes `&self` by ref
    }

    pub fn insert(&mut self, area: A, data: T) -> Result<()> {
        self.insert_within_impl(None, area, data)
    }

    pub fn insert_within(
        &mut self,
        bounds: &Rect,
        area: A,
        data: T,
    ) -> Result<()> {
        self.insert_within_impl(Some(bounds), area, data)
    }

    pub fn move_within_by_from(
        &mut self,
        bounds: &Rect,
        v: IVec2,
        p: &Point,
    ) -> Result<()>
    where
        A: core::ops::Add<IVec2, Output = A>,
    {
        let Entity { area, data } = self
            .extract_at(p)
            .ok_or_else(|| err!("No element found at {p}"))?;

        self.insert_within(bounds, area + v, data)
    }

    pub fn move_within_by_from_any<'a>(
        &mut self,
        bounds: &Rect,
        v: IVec2,
        points: impl IntoIterator<Item = &'a Point>,
    ) -> Result<()>
    where
        A: core::ops::Add<IVec2, Output = A>,
    {
        let mut errs =
            ErrorStash::new(|| "Failed to move one or more entities");

        // We have to extract ALL elements first in order to avoid collisions.
        let extracted: Vec<_> = self.extract_at_any(points).collect();

        for Entity { area, data } in extracted {
            self.insert_within(bounds, area + v, data)
                .or_stash(&mut errs);
        }

        errs.into()
    }

    pub fn extract(&mut self, e: &Entity<A, T>) -> Option<Entity<A, T>> {
        let id = self.get_at_exactly_impl(&e.area)?;
        Some(self.extract_by_id_or_panic(id))
    }

    pub fn extract_at(&mut self, p: &Point) -> Option<Entity<A, T>> {
        let id = self.get_at_impl(p)?;
        Some(self.extract_by_id_or_panic(id))
    }

    pub fn extract_at_any<'a, I>(
        &mut self,
        points: I,
    ) -> impl Iterator<Item = Entity<A, T>> + use<'_, 'a, A, T, I>
    where
        I: IntoIterator<Item = &'a Point>,
    {
        points
            .into_iter()
            .flat_map(|p| self.extract_at(p))
    }

    pub fn extract_exactly_one(&mut self, data: &T) -> Result<Entity<A, T>>
    where
        T: PartialEq,
    {
        self.find_exactly_one_impl(data)
            .and_then(|id| self.extract_by_id(id))
    }

    fn get_at_impl(&self, p: &Point) -> Option<usize> {
        self.point_to_id.get(p).copied()
    }

    fn get_at_exactly_impl(&self, a: &A) -> Option<usize> {
        self.area_to_id.get(a).copied()
    }

    fn get_by_id(&self, id: usize) -> Result<&Entity<A, T>> {
        self.id_to_entity
            .get(&id)
            .ok_or_else(|| -> Error { err!("Failed to find entity #{id}") })
            .or_wrap_with(|| MSG_INCONSISTENT)
    }

    fn get_by_id_or_panic(&self, id: usize) -> &Entity<A, T> {
        self.get_by_id(id).unwrap()
    }

    fn find_all_impl<'a, 'b>(
        &'a self,
        data: &'b T,
    ) -> impl Iterator<Item = usize> + use<'a, 'b, A, T>
    where
        T: PartialEq,
    {
        self.id_to_entity
            .iter()
            .filter(move |(_, e)| &e.data == data)
            .map(|(&i, _)| i)
    }

    fn find_exactly_one_impl(&self, data: &T) -> Result<usize>
    where
        T: PartialEq,
    {
        use itertools::Itertools;

        self.find_all_impl(data)
            .exactly_one()
            .map_err(|iter| {
                let n = iter.count();
                err!("Expected to find exactly one element, got {n}")
            })
    }

    fn insert_within_impl(
        &mut self,
        bounds: Option<&Rect>,
        area: A,
        data: T,
    ) -> Result<()> {
        let area_as_points: Vec<Point> = area.clone().into();
        self.ensure_insertable(bounds, &area_as_points)?;

        for p in area_as_points {
            self.point_to_id.insert(p, self.next_id);
        }

        self.area_to_id
            .insert(area.clone(), self.next_id);

        let e = Entity { area, data };
        self.id_to_entity
            .insert(self.next_id, e);

        self.next_id += 1;

        Ok(())
    }

    fn extract_by_id(&mut self, id: usize) -> Result<Entity<A, T>> {
        let err = || -> Error { err!("Failed to find entity #{id}") };

        let e = self
            .id_to_entity
            .remove(&id)
            .ok_or_else(err)
            .or_wrap_with(|| MSG_INCONSISTENT)?;

        self.area_to_id
            .remove(&e.area)
            .ok_or_else(err)
            .or_wrap_with(|| MSG_INCONSISTENT)?;

        for p in e.area.clone().into() {
            self.point_to_id
                .remove(&p)
                .ok_or_else(|| -> Error {
                    err!("Object is already (partially?) missing at {p}")
                })
                .or_wrap_with(|| MSG_INCONSISTENT)?;
        }

        Ok(e)
    }

    fn extract_by_id_or_panic(&mut self, id: usize) -> Entity<A, T> {
        self.extract_by_id(id).unwrap()
    }

    fn ensure_insertable(
        &mut self,
        bounds: Option<&Rect>,
        points: &[Point],
    ) -> Result<()> {
        let mut errs = ErrorStash::new(|| "Cannot insert data into grid");

        for p in points {
            if let Some(bounds) = bounds {
                if !bounds.contains(p) {
                    errs.push(format!("Out of bounds: {p}"));
                }
            }

            if self.point_to_id.contains_key(p) {
                errs.push(format!("Collision: {p}"));
            }
        }

        errs.into()
    }
}

impl<A, T> Clone for UnboundedGrid<A, T>
where
    A: Clone + Hash + Eq,
    T: Clone,
{
    /// Implements [`Clone`] explicitly because
    /// [`Entity`] must not be [`Clone`].
    ///
    /// We cannot make [`Entity`] implement [`Clone`] because
    /// users would be able to call `let entity = grid.get_at(point).clone()`,
    /// dropping the shared reference into `grid` and thus making `grid`
    /// mutable again, and then modify `grid` and all of its entities.
    /// The cloned and now outdated `entity` could later be passed by reference
    /// into `grid`, tricking `grid` into violating invariants.
    fn clone(&self) -> Self {
        let next_id = self.next_id;
        let area_to_id = self.area_to_id.clone();
        let point_to_id = self.point_to_id.clone();
        let id_to_entity = self
            .id_to_entity
            .iter()
            .map(|(id, Entity { area, data })| {
                let area = area.clone();
                let data = data.clone();
                (*id, Entity { area, data })
            })
            .collect();

        Self {
            next_id,
            area_to_id,
            point_to_id,
            id_to_entity,
        }
    }
}

const RENDER_UNIT_AS_CHAR: char = '#';

impl RenderOn<Point> for () {
    fn render_on(&self, _: &Point) -> Result<Vec<Vec<char>>> {
        Ok(vec![vec![RENDER_UNIT_AS_CHAR]])
    }
}

impl RenderOn<Rect> for () {
    fn render_on(&self, rect: &Rect) -> Result<Vec<Vec<char>>> {
        let y = rect.len().y();
        let x = rect.len().x();
        Ok(vec![vec![RENDER_UNIT_AS_CHAR; x]; y])
    }
}

impl<A, T> fmt::Display for Grid<A, T>
where
    A: Position + Hash + Eq,
    T: RenderOn<A>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use itertools::Itertools;

        let origin = self.bounds.pos();
        let y_len = self.bounds.len().y();
        let x_len = self.bounds.len().x();

        let empty_line = vec![' '; x_len];
        let mut output = vec![empty_line; y_len];

        let pixels = self
            .into_iter()
            .map(|Entity { area, data }| -> Result<_> {
                let pixels = data.render_on(area)?;

                let offs: IVec2 = area.position() - origin;
                let offs: UVec2 = UVec2::try_from(offs)?;
                let (y0, x0) = (offs.y(), offs.x());

                Ok(pixels
                    .into_iter()
                    .enumerate()
                    .flat_map(|(y, cols)| {
                        cols.into_iter()
                            .enumerate()
                            .map(move |(x, char)| (y, x, char))
                    })
                    .map(move |(y, x, char)| (y0 + y, x0 + x, char)))
            })
            .flatten_ok();

        for pixel in pixels {
            let (y, x, char) = pixel.map_err(|_| fmt::Error)?;
            output[y][x] = char;
        }

        for line in output {
            let line: String = line.into_iter().collect();
            writeln!(f, "{}", line)?;
        }

        Ok(())
    }
}

impl<'a, S, T> IntoIterator for &'a Grid<S, T>
where
    S: Hash + Eq,
{
    type IntoIter = hash_map::Values<'a, usize, Entity<S, T>>;
    type Item = &'a Entity<S, T>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.grid).into_iter()
    }
}

impl<S, T> IntoIterator for Grid<S, T>
where
    S: Hash + Eq,
{
    type IntoIter = hash_map::IntoValues<usize, Entity<S, T>>;
    type Item = Entity<S, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.grid.into_iter()
    }
}

impl<'a, S, T> IntoIterator for &'a UnboundedGrid<S, T>
where
    S: Hash + Eq,
{
    type IntoIter = hash_map::Values<'a, usize, Entity<S, T>>;
    type Item = &'a Entity<S, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.id_to_entity.values()
    }
}

impl<S, T> IntoIterator for UnboundedGrid<S, T>
where
    S: Hash + Eq,
{
    type IntoIter = hash_map::IntoValues<usize, Entity<S, T>>;
    type Item = Entity<S, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.id_to_entity.into_values()
    }
}
