pub mod geometry;
pub mod charges;
pub mod voltmeter; 

pub trait Drawable {
    fn draw(&self);
}


type ImplIteratorMut<'a, Item> =
std::iter::Chain<
    std::slice::IterMut<'a, Item>,
    std::slice::IterMut<'a, Item>,
>
;
pub trait SplitOneMut {
    type Item;

    fn split_one_mut (
        &'_ mut self,
        i: usize,
    ) -> (&'_ mut Self::Item, ImplIteratorMut<'_, Self::Item>);
}

impl<T> SplitOneMut for [T] {
    type Item = T;

    fn split_one_mut (
        &'_ mut self,
        i: usize,
    ) -> (&'_ mut Self::Item, ImplIteratorMut<'_, Self::Item>)
    {
        let (prev, current_and_end) = self.split_at_mut(i);
        let (current, end) = current_and_end.split_at_mut(1);
        (
            &mut current[0],
            prev.iter_mut().chain(end),
        )
    }
}


