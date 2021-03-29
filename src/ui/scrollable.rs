pub trait Scrollable {
	fn selected(&self) -> usize;
	fn len(&self) -> usize;
	fn set_selected(&mut self, selected: usize) -> bool;
	fn element_height(&self, index: usize) -> u16;
	fn visible_range(&self, height: u16) -> Box<dyn Iterator<Item = usize>>;
	fn visible_start_end(&self, height: u16) -> (usize, usize);
	fn up(&mut self, how_much: usize);
	fn down(&mut self, how_much: usize);
}

#[macro_export]
macro_rules! scrollable {
    ($x:ty, $($y:item),*) => {
        impl Scrollable for $x {
            fn up(&mut self, how_much: usize) {
                let how_much = how_much % self.len();

                self.set_selected((self.len() + self.selected() - how_much) % self.len());
            }
            fn down(&mut self, how_much: usize) {
                let how_much = how_much % self.len();

                self.set_selected((how_much + self.selected()) % self.len());
            }
            fn visible_range(&self, height: u16) -> Box<dyn Iterator<Item = usize>> {
                let (a, b) = self.visible_start_end(height);

                Box::new((a..b).into_iter())
            }
            fn visible_start_end(&self, height: u16) -> (usize, usize) {
                let mut last_first = 0;
                let mut current_height = 0;
                let mut i = 0;
                while i < self.len() {
                    let eh =  self.element_height(i);
                    if current_height + eh > height {
                        if i > self.selected() {
                            break;
                        }
                        current_height = eh;
                        last_first = i;
                    } else {
                        current_height += eh;
                    }

                    i += 1;
                }

                (last_first, i)
            }
            $($y)*
        }
    }
}
