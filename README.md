[![Build Status](https://travis-ci.org/danylaporte/consolidated_map.svg?branch=master)](https://travis-ci.org/danylaporte/consolidated_map)

A map to give all the children associated with an item.

## Documentation
[API Documentation](https://danylaporte.github.io/consolidated_map/consolidated_map)

## Example

```rust
use consolidated_map::ConsolidatedMapBuilder;

fn main() {
    let mut builder = ConsolidatedMapBuilder::new();
    
    // associate the child 20 with the parent 10.
    builder.insert(10usize, 20);
    
    // associate the child 30 with the parent 20.
    builder.insert(20, 30);
    
    // build the ConsolidatedMap
    let map = builder.build();
    
    // the parent 10 should have the children 20 and 30.
    assert_eq!(map.children(10).collect::<Vec<_>>(), vec![20, 30]);
    
    // the parent 20 should have the children 30.
    assert_eq!(map.children(20).collect::<Vec<_>>(), vec![30]);
    
    // the parent 30 does not have any children.
    assert!(map.children(30).collect::<Vec<_>>().is_empty());

    // consolidated children contains also the key item.
    assert_eq!(map.consolidated(10).collect::<Vec<_>>(), vec![10, 20, 30]);

    // if the item has not been inserted, the consolidated
    // function returns only the item.
    assert_eq!(map.consolidated(5).collect::<Vec<_>>(), vec![5]);
}
```

## License

Dual-licensed to be compatible with the Rust project.

Licensed under the Apache License, Version 2.0
[http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0) or the MIT license
[http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT), at your
option. This file may not be copied, modified, or distributed
except according to those terms.