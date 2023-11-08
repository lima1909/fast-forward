use crate::index::indices::KeyIndex;

// ------------
// --- READ ---
// ------------
pub(crate) trait KeyIndexOptionRead<I, X>
where
    I: KeyIndex<X>,
{
    type Output;

    fn contains(&self, is_negativ: bool) -> bool;
    fn get(&self, is_negativ: bool) -> &[X];

    fn get_opt(&self, _: bool) -> &Option<I> {
        &None
    }
    fn map_to_position(&self, _: usize) -> Option<Self::Output> {
        None
    }
}

impl<I, X> KeyIndexOptionRead<I, X> for Option<I>
where
    I: KeyIndex<X>,
{
    type Output = usize;

    fn contains(&self, _: bool) -> bool {
        self.is_some()
    }

    fn get(&self, _: bool) -> &[X] {
        self.as_ref().map_or(&[], |i| i.as_slice())
    }

    fn get_opt(&self, _: bool) -> &Option<I> {
        self
    }

    fn map_to_position(&self, pos: usize) -> Option<Self::Output> {
        self.as_ref().map(|_| pos)
    }
}

impl<I, X> KeyIndexOptionRead<I, X> for Option<&I>
where
    I: KeyIndex<X>,
{
    type Output = usize;

    fn contains(&self, _: bool) -> bool {
        self.is_some()
    }

    fn get(&self, _: bool) -> &[X] {
        self.as_ref().map_or(&[], |i| (*i).as_slice())
    }
}

impl<I, X> KeyIndexOptionRead<I, X> for (Option<I>, Option<I>)
where
    I: KeyIndex<X>,
{
    type Output = (Option<usize>, Option<usize>);

    fn contains(&self, is_negativ: bool) -> bool {
        self.get_opt(is_negativ).is_some()
    }

    fn get(&self, is_negativ: bool) -> &[X] {
        self.get_opt(is_negativ).get(is_negativ)
    }

    fn get_opt(&self, is_negativ: bool) -> &Option<I> {
        if is_negativ {
            &self.0
        } else {
            &self.1
        }
    }

    fn map_to_position(&self, pos: usize) -> Option<Self::Output> {
        if self.0.is_none() && self.1.is_none() {
            None
        } else {
            Some((self.0.map_to_position(pos), self.1.map_to_position(pos)))
        }
    }
}

// -------------
// --- WRITE ---
// -------------
pub(crate) trait KeyIndexOptionWrite<I, X>: Clone + Default
where
    I: KeyIndex<X>,
{
    fn set(&mut self, is_negativ: bool, index: X);
    fn delete(&mut self, is_negativ: bool, index: &X);

    fn get_opt_mut(&mut self, _: bool) -> &mut Option<I>;
}

impl<I, X> KeyIndexOptionWrite<I, X> for Option<I>
where
    I: KeyIndex<X> + Clone,
{
    fn set(&mut self, _: bool, index: X) {
        match self {
            Some(idx) => idx.add(index),
            None => *self = Some(I::new(index)),
        };
    }

    fn delete(&mut self, _: bool, index: &X) {
        if let Some(rm_idx) = self {
            if rm_idx.remove(index) {
                *self = None;
            }
        }
    }

    fn get_opt_mut(&mut self, _: bool) -> &mut Option<I> {
        self
    }
}

impl<I, X> KeyIndexOptionWrite<I, X> for (Option<I>, Option<I>)
where
    I: KeyIndex<X> + Clone,
{
    fn set(&mut self, is_negativ: bool, index: X) {
        self.get_opt_mut(is_negativ).set(is_negativ, index);
    }

    fn delete(&mut self, is_negativ: bool, index: &X) {
        self.get_opt_mut(is_negativ).delete(is_negativ, index);
    }

    fn get_opt_mut(&mut self, is_negativ: bool) -> &mut Option<I> {
        if is_negativ {
            &mut self.0
        } else {
            &mut self.1
        }
    }
}
