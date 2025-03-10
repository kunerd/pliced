use iced::Rectangle;
use ordered_float::OrderedFloat;

use std::collections::BTreeMap;

type BTreeMapFloat<V> = BTreeMap<OrderedFloat<f32>, V>;

pub struct Items<SeriesId, ItemId>(BTreeMapFloat<BTreeMapFloat<(SeriesId, ItemId)>>);

impl<SeriesId> Items<SeriesId, usize>
where
    SeriesId: Clone,
{
    pub fn add_series(&mut self, id: SeriesId, series: &[iced::Point]) {
        for (index, point) in series.iter().enumerate() {
            self.0.entry(OrderedFloat(point.x)).or_insert_with(|| {
                let mut map = BTreeMap::new();
                map.insert(OrderedFloat(point.y), (id.clone(), index));
                map
            });
        }
    }

    pub fn collision(&self, rect: Rectangle) -> Vec<(SeriesId, usize)> {
        let range = OrderedFloat(rect.x)..OrderedFloat(rect.x + rect.width);

        let mut items = vec![];
        for (_, bucket) in self.0.range(range) {
            let range = OrderedFloat(rect.y)..OrderedFloat(rect.y + rect.height);

            let item_list = bucket
                .range(range)
                .map(|(_key, (id, index))| (id.clone(), *index));

            items.extend(item_list);
        }

        items
    }
}

impl<SeriesId, ItemId> Default for Items<SeriesId, ItemId> {
    fn default() -> Self {
        Self(BTreeMapFloat::new())
    }
}
