use crate::index::indices::KeyIndex;

// ------------
// --- READ ---
// ------------
pub(crate) trait KeyIndexOptionRead<I, X>
where
    I: KeyIndex<X>,
{
    fn contains(&self, is_negativ: bool) -> bool;
    fn get(&self, is_negativ: bool) -> &[X];
}

impl<I, X> KeyIndexOptionRead<I, X> for Option<I>
where
    I: KeyIndex<X>,
{
    fn contains(&self, _: bool) -> bool {
        self.is_some()
    }

    fn get(&self, _: bool) -> &[X] {
        self.as_ref().map_or(&[], |i| i.as_slice())
    }
}

impl<I, X> KeyIndexOptionRead<I, X> for Option<&I>
where
    I: KeyIndex<X>,
{
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
    fn contains(&self, is_negativ: bool) -> bool {
        if is_negativ { &self.0 } else { &self.1 }.is_some()
    }

    fn get(&self, is_negativ: bool) -> &[X] {
        if is_negativ { &self.0 } else { &self.1 }.get(is_negativ)
    }
}

impl<I, X> KeyIndexOptionRead<I, X> for (Option<&I>, Option<&I>)
where
    I: KeyIndex<X>,
{
    fn contains(&self, is_negativ: bool) -> bool {
        if is_negativ { self.0 } else { self.1 }.is_some()
    }

    fn get(&self, is_negativ: bool) -> &[X] {
        if is_negativ { self.0 } else { self.1 }
            .as_ref()
            .map_or(&[], |i| (*i).as_slice())
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
}

impl<I, X> KeyIndexOptionWrite<I, X> for (Option<I>, Option<I>)
where
    I: KeyIndex<X> + Clone,
{
    fn set(&mut self, is_negativ: bool, index: X) {
        if is_negativ { &mut self.0 } else { &mut self.1 }.set(is_negativ, index);
    }

    fn delete(&mut self, is_negativ: bool, index: &X) {
        if is_negativ { &mut self.0 } else { &mut self.1 }.delete(is_negativ, index);
    }
}

// ------------
// --- Meta ---
// ------------
pub(crate) trait KeyIndexOptionMeta<I, X>
where
    I: KeyIndex<X>,
{
    type Output;

    fn map_to_position(&self, _: usize) -> Option<Self::Output>;
}

impl<I, X> KeyIndexOptionMeta<I, X> for Option<I>
where
    I: KeyIndex<X>,
{
    type Output = usize;

    fn map_to_position(&self, pos: usize) -> Option<Self::Output> {
        self.as_ref().map(|_| pos)
    }
}

impl<I, X> KeyIndexOptionMeta<I, X> for (Option<I>, Option<I>)
where
    I: KeyIndex<X>,
{
    type Output = (Option<usize>, Option<usize>);

    fn map_to_position(&self, pos: usize) -> Option<Self::Output> {
        if self.0.is_none() && self.1.is_none() {
            None
        } else {
            Some((self.0.map_to_position(pos), self.1.map_to_position(pos)))
        }
    }
}
