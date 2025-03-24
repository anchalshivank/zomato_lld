#![allow(warnings)]
use std::fmt::Debug;
use std::collections::HashMap;

#[derive(Debug)]
enum CustomError {
    NotificationError,
    PaymentError,
    OrderError,
    RiderError,
    Other(String),
}

// Location
#[derive(Clone, Debug)]
struct Location(i32, i32);

impl Location {
    fn distance_to(&self, other: &Location) -> f64 {
        let dx = (self.0 - other.0) as f64;
        let dy = (self.1 - other.1) as f64;
        (dx * dx + dy * dy).sqrt()
    }
}

// User
#[derive(Debug)]
struct User {
    id: String,
    name: String,
    location: Location,
}

impl User {
    fn new(id: &str, name: &str, location: Location) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            location,
        }
    }
}

// Notification Instrument
trait NotificationInstrument: Debug {
    fn notify(&self, message: &str) -> Result<(), CustomError>;
}

#[derive(Debug)]
struct Email {
    id: String,
}

impl Email {
    fn new(id: &str) -> Self {
        Self { id: id.to_string() }
    }
}

impl NotificationInstrument for Email {
    fn notify(&self, message: &str) -> Result<(), CustomError> {
        println!("SENDING NOTIFICATION TO {}: {}", self.id, message);
        Ok(())
    }
}

// Notification Manager
#[derive(Debug)]
struct NotificationManager {
    nm: HashMap<String, Box<dyn NotificationInstrument>>,
}

impl NotificationManager {
    fn new() -> Self {
        Self { nm: HashMap::new() }
    }
    fn attach(&mut self, user: &User, instrument: Box<dyn NotificationInstrument>) {
        self.nm.insert(user.id.clone(), instrument);
    }
    fn get(&mut self, user: &User) -> Option<&mut Box<dyn NotificationInstrument>> {
        self.nm.get_mut(&user.id)
    }
}

// Payment Instrument
trait PaymentInstrument: Debug {
    fn pay(&mut self, amount: usize) -> Result<usize, CustomError>;
}

#[derive(Debug)]
struct Gpay {
    id: String,
    balance: usize,
}

impl Gpay {
    fn new(id: &str, balance: usize) -> Self {
        Self { id: id.to_string(), balance }
    }
}

impl PaymentInstrument for Gpay {
    fn pay(&mut self, amount: usize) -> Result<usize, CustomError> {
        if self.balance >= amount {
            self.balance -= amount;
            Ok(self.balance)
        } else {
            Err(CustomError::PaymentError)
        }
    }
}

// Payment Manager
#[derive(Debug)]
struct PaymentManager {
    pm: HashMap<String, Box<dyn PaymentInstrument>>,
}

impl PaymentManager {
    fn new() -> Self {
        Self { pm: HashMap::new() }
    }
    fn attach(&mut self, user: &User, instrument: Box<dyn PaymentInstrument>) {
        self.pm.insert(user.id.clone(), instrument);
    }
    fn get(&mut self, user: &User) -> Option<&mut Box<dyn PaymentInstrument>> {
        self.pm.get_mut(&user.id)
    }
}

// Cart Instrument
trait CartInstrument: Debug {
    fn add(&mut self, item: &Item);
    fn remove(&mut self, item: &Item);
    fn get_items(&self) -> &HashMap<String, usize>;
    fn clear(&mut self);
}

#[derive(Debug)]
struct Cart {
    items: HashMap<String, usize>,
}

impl Cart {
    fn new() -> Self {
        Self { items: HashMap::new() }
    }
}

impl CartInstrument for Cart {
    fn add(&mut self, item: &Item) {
        *self.items.entry(item.id.clone()).or_insert(0) += 1;
    }
    
    fn remove(&mut self, item: &Item) {
        if let Some(quantity) = self.items.get_mut(&item.id) {
            if *quantity > 1 {
                *quantity -= 1;
            } else {
                self.items.remove(&item.id);
            }
        }
    }
    
    fn get_items(&self) -> &HashMap<String, usize> {
        &self.items
    }
    
    fn clear(&mut self) {
      self.items.clear();
    }
}

// Cart Manager
#[derive(Debug)]
struct CartManager {
    cm: HashMap<String, Box<dyn CartInstrument>>,
}

impl CartManager {
    fn new() -> Self {
        Self { cm: HashMap::new() }
    }
    fn attach(&mut self, user: &User, cart: Box<dyn CartInstrument>) {
        self.cm.insert(user.id.clone(), cart);
    }
    fn get(&mut self, user: &User) -> Option<&mut Box<dyn CartInstrument>> {
        self.cm.get_mut(&user.id)
    }
}

// Restaurant
#[derive(Debug)]
struct Restaurant {
    id: String,
    name: String,
    location: Location,
    menu: HashMap<String, usize>, // item_id -> price
}

impl Restaurant {
    fn new(id: &str, name: &str, location: Location, menu: HashMap<String, usize>) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            location,
            menu,
        }
    }
}

// Item
#[derive(Debug)]
struct Item {
    id: String,
    price: usize,
}

impl Item {
    fn new(id: &str, price: usize) -> Self {
        Self { id: id.to_string(), price }
    }
}

// Rider
#[derive(Debug)]
struct Rider {
    id: String,
    location: Option<Location>,
    target_location: Option<Location>,
    is_available: bool,
}

impl Rider {
    fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            location: None,
            target_location: None,
            is_available: true,
        }
    }
    
    fn update(&mut self, location: Location) {
        self.location = Some(location);
    }
    
    fn accept_ride(&mut self, target_location: Location) {
        self.target_location = Some(target_location);
        self.is_available = false;
    }
}

// Rider Matching Service
#[derive(Debug)]
struct RiderMatchingService {
    riders: Vec<Rider>,
}

impl RiderMatchingService {
    fn new() -> Self {
        Self { riders: Vec::new() }
    }
    
    fn push(&mut self, rider: Rider) {
        self.riders.push(rider);
    }
    
    fn match_rider(&mut self, target_location: &Location) -> Result<&mut Rider, CustomError> {
        let available_riders: Vec<(usize, &mut Rider)> = self.riders.iter_mut()
            .enumerate()
            .filter(|(_, rider)| rider.is_available)
            .collect();
        
        if available_riders.is_empty() {
            println!("No rider found");
            return Err(CustomError::RiderError);
        }
        
        let mut min_idx = available_riders[0].0;
        let mut min_d = std::f64::MAX;
        
        for (idx, rider) in available_riders.iter() {
            if let Some(loc) = &rider.location {
                let c = loc.distance_to(target_location);
                if c < min_d {
                    min_d = c;
                    min_idx = *idx;
                }
            }
        }
        
        let closest_rider = &mut self.riders[min_idx];
        closest_rider.accept_ride(target_location.clone());
        Ok(closest_rider)
    }
}

// Zomato Service
#[derive(Debug)]
struct Zomato {
    notification_manager: NotificationManager,
    payment_manager: PaymentManager,
    cart_manager: CartManager,
    rider_service: RiderMatchingService,
    restaurants: HashMap<String, Restaurant>,
}

impl Zomato {
    fn new() -> Self {
        Self {
            notification_manager: NotificationManager::new(),
            payment_manager: PaymentManager::new(),
            cart_manager: CartManager::new(),
            rider_service: RiderMatchingService::new(),
            restaurants: HashMap::new(),
        }
    }

    fn add_restaurant(&mut self, restaurant: Restaurant) {
        self.restaurants.insert(restaurant.id.clone(), restaurant);
    }

    fn add_to_cart(&mut self, user: &User, item: &Item) -> Result<(), CustomError> {
        let cart = if let Some(cart) = self.cart_manager.get(user) {
            cart
        } else {
            let new_cart = Box::new(Cart::new());
            self.cart_manager.attach(user, new_cart);
            self.cart_manager.get(user).unwrap()
        };
        cart.add(item);
        Ok(())
    }

    fn process_order(&mut self, user: &User, restaurant_id: &str) -> Result<(), CustomError> {
        let mut cart = self.cart_manager.get(user)
            .ok_or(CustomError::OrderError)?;
        
        let restaurant = self.restaurants.get(restaurant_id)
            .ok_or(CustomError::OrderError)?;
        
        let total: usize = cart.get_items().iter()
            .map(|(item_id, &qty)| {
                restaurant.menu.get(item_id)
                    .ok_or(CustomError::OrderError)
                    .map(|price| price * qty)
                    .unwrap_or(0)
            })
            .sum();

        let balance = self.payment_manager.get(user)
            .ok_or(CustomError::PaymentError)?
            .pay(total)?;

        let rider = self.rider_service.match_rider(&user.location)?;

        self.notification_manager.get(user)
            .ok_or(CustomError::NotificationError)?
            .notify(&format!(
                "Order of ₹{} from {} processed. Rider {} assigned. Balance: ₹{}", 
                total, restaurant.name, rider.id, balance
            ))?;

        cart.clear();
        Ok(())
    }
}

fn main() {
    let mut zomato = Zomato::new();

    // Setup restaurant
    let mut menu = HashMap::new();
    let item1 = Item::new("1", 12);
    let item2 = Item::new("2", 14);
    menu.insert(item1.id.clone(), item1.price);
    menu.insert(item2.id.clone(), item2.price);
    let restaurant = Restaurant::new("1", "Karnot Dhaba", Location(1, 1), menu);
    zomato.add_restaurant(restaurant);

    // Setup users
    let user1 = User::new("1", "Shivank", Location(1, 2));
    zomato.notification_manager.attach(&user1, Box::new(Email::new("shivank@gmail.com")));
    zomato.payment_manager.attach(&user1, Box::new(Gpay::new("shivank", 100)));

    let user2 = User::new("2", "Ajay", Location(1, 3));
    zomato.notification_manager.attach(&user2, Box::new(Email::new("ajay@gmail.com")));
    zomato.payment_manager.attach(&user2, Box::new(Gpay::new("ajay", 150)));

    // Setup riders
    let mut rider1 = Rider::new("r1");
    rider1.update(Location(2, 2));
    let mut rider2 = Rider::new("r2");
    rider2.update(Location(3, 3));
    zomato.rider_service.push(rider1);
    zomato.rider_service.push(rider2);

    // Process order for user1
    println!("=== Order for {} ===", user1.name);
    zomato.add_to_cart(&user1, &item1).unwrap();
    zomato.add_to_cart(&user1, &item2).unwrap();
    match zomato.process_order(&user1, "1") {
        Ok(()) => println!("Order completed successfully"),
        Err(e) => println!("Order failed: {:?}", e),
    }

    // Process order for user2
    println!("\n=== Order for {} ===", user2.name);
    zomato.add_to_cart(&user2, &item1).unwrap();
    zomato.add_to_cart(&user2, &item1).unwrap();
    match zomato.process_order(&user2, "1") {
        Ok(()) => println!("Order completed successfully"),
        Err(e) => println!("Order failed: {:?}", e),
    }

    println!("\nFinal state: {:?}", zomato);
}
