use skipdb::comparable::ComparableDb;

#[derive(Debug)]
struct Person {
  name: String,
  hobby: String,
  age: u8,
}

fn main() {
  let db: ComparableDb<u64, Person> = ComparableDb::new();

  {
    let alice = Person {
      name: "Alice".to_string(),
      hobby: "swim".to_string(),
      age: 20,
    };
    let bob = Person {
      name: "Bob".to_string(),
      hobby: "run".to_string(),
      age: 30,
    };

    let mut txn = db.write();
    txn.insert(1, alice).unwrap();
    txn.insert(2, bob).unwrap();

    {
      let alice = txn.get(&1).unwrap().unwrap();
      assert_eq!(alice.value().name, "Alice");
      assert_eq!(alice.value().age, 20);
      assert_eq!(alice.value().hobby, "swim");
    }

    {
      // output:
      // 1: Person { name: "Alice", hobby: "swim", age: 20 }
      // 2: Person { name: "Bob", hobby: "run", age: 30 }
      for ent in txn.iter().unwrap() {
        println!("{}: {:?}", ent.key(), ent.value());
      }

      // output:
      // 2: Person { name: "Bob", hobby: "run", age: 30 }
      // 1: Person { name: "Alice", hobby: "swim", age: 20 }
      for ent in txn.iter_rev().unwrap() {
        println!("{}: {:?}", ent.key(), ent.value());
      }

      // output:
      // 2: Person { name: "Bob", hobby: "run", age: 30 }
      for ent in txn.range(1..).unwrap() {
        println!("{}: {:?}", ent.key(), ent.value());
      }

      // output:
      // 1: Person { name: "Alice", hobby: "swim", age: 20 }
      for ent in txn.range_rev(..2).unwrap() {
        println!("{}: {:?}", ent.key(), ent.value());
      }
    }

    txn.commit().unwrap();
  }

  {
    let txn = db.read();
    let alice = txn.get(&1).unwrap();
    assert_eq!(alice.value().name, "Alice");
    assert_eq!(alice.value().age, 20);
    assert_eq!(alice.value().hobby, "swim");

    let bob = txn.get(&2).unwrap();
    assert_eq!(bob.value().name, "Bob");
    assert_eq!(bob.value().age, 30);
    assert_eq!(bob.value().hobby, "run");

    // output:
    // 1: Person { name: "Alice", hobby: "swim", age: 20 }
    // 2: Person { name: "Bob", hobby: "run", age: 30 }
    for ent in txn.iter() {
      println!("{}: {:?}", ent.key(), ent.value());
    }

    // output:
    // 2: Person { name: "Bob", hobby: "run", age: 30 }
    // 1: Person { name: "Alice", hobby: "swim", age: 20 }
    for ent in txn.iter_rev() {
      println!("{}: {:?}", ent.key(), ent.value());
    }

    // output:
    // 2: Person { name: "Bob", hobby: "run", age: 30 }
    for ent in txn.range(1..) {
      println!("{}: {:?}", ent.key(), ent.value());
    }

    // output:
    // 1: Person { name: "Alice", hobby: "swim", age: 20 }
    for ent in txn.range(..2) {
      println!("{}: {:?}", ent.key(), ent.value());
    }
  }
}
