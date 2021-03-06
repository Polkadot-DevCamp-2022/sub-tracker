# Asset Tracking System and Verification on Substrate

## Shipment Tracking and Verification on Substrate

The aim of this project is to build a shipment tracking and verification blockchain using substrate. The Minimum Viable Product (MVP) will have the functionality to create a shipment with a unique ID and a predefined static route. The verification component will consist of generating a secret key for the next transit point of the shipment at every transit point.

### Example

Let there be:

- N transit points { TP1, TP2, ..., TPN}
- A shipment with:
  - route = { TP4, TP8, TP11 }
  - uid = 112
  - key = X
  - owner = TP8
- Package with a physical tag and seal that can be scanned to get the key

When TP8 receives the package, the following actions will be carried out:

1. TP8 calls an update function that accepts the package uid and scanned key
2. TP8 verifies that the key and the owner information is correct
3. TP8 generates a new key and update the owner of the package to TP11

When TP11 receives the package, the following actions will be carried out:

1. TP11 calls an update function that accepts the package uid and scanned key
2. TP11 verifies that the key and the owner information is correct
3. TP11 marks the package as shipped and set keys/owner to None

## Blockchain Overview

Slides: <https://docs.google.com/presentation/d/1U6y1i2ZFTuHamG2VXajVnKJHbgnUU92Fg4DP0efLqpY/edit#slide=id.g12ebce7dbdb_0_991>

### Adding Transit Nodes 

```
pub fn create_new_transit_node(
    origin: OriginFor<T>,
    transit_node: T::AccountId,
    neighbours: BoundedVec<(T::AccountId, u32), T::MaxSize>) 
 ```

New nodes can only be added via Sudo. Adding of nodes require two input arguments:
1. **Account Id** of the transit node to be added
2. **Vector** of **Account Id** and **Cost** pairings. This vector defines the cost of the route between the new node and existing nodes. i.e. The neighbours of the node to be added<br>

*Note: This function will fail on multiple scenarios:
    a. Call is not made by sudo
    b. Node has already been added
    c. Account Id of node to be added is included in the vector
    d. Account Id of any node in vector has not been added as a transit node*

### Removing Transit Nodes

```
pub fn update_neighbour(
    origin: OriginFor<T>,
    node1: T::AccountId,
    node2: T::AccountId,
    cost: u32)
```

Nodes can only be removed via Sudo. Removing of nodes require one input argument:
1. **Account Id** of transit node to be removed<br>

*Note: This function will fail on multiple scenarios:
    a. Call is not made by sudo
    b. Node has not been added as a transit node*

### Updating Neighbour Costs

```
pub fn remove_transit_node(origin: OriginFor<T>, transit_node: T::AccountId)
```

Updating of neighbouring nodes cost can only be done via Sudo. Updating of neighbours require three input arguments:
1. **Account Id** of first transit node
2. **Account Id** of second transit node
3. **Cost** of route between the two specified nodes<br>

*Note: This function will fail if any one of the account ids have not been added as a transit node*

### Creating Shipments

```
pub fn create_shipment(origin: OriginFor<T>, destination: T::AccountId)
```


Shipments can be created via signed transactions by any transit node. Shipment routes are defaulted to begin at the transit node that created the shipment. Creating of shipments require one input argument:
1. **Destination** of the shipment. The most cost efficient route will then be computed based on the source and destination of the shipment

### Updating Shipments

```
pub fn update_shipment(origin: OriginFor<T>, shipment_uid: u64, key: [u8; 16])
```

Shipments can be updated via signed transactions by the current receiver of a shipment. Updating of shipments requires two input argument:
1. **Shipment UID**
2. **key**<br>

In a real world scenario , key can be decoded from the machine readable cde embedded into the package. For testing, we can use the getter shipment_uid_to_key() to get the key. This function accepts the shipment uid and returns the key. 

*Note: This function will fail on multiple scenarios:
    a. Call is not made by current receiver of shipment
    b. Shipment UID could not be found*
    
   
### Tracking Shipments

Shipments can be tracked by the UID of each shipment. Getter function uid_to_shipment() accepts the uid and returns the shipment struct which contains all the updated information about the package.

## Usage

### Backend

```
cd substrate-node-template
cargo build --release
cargo run --release -- --dev --tmp
```

### Frontend

The frontend is not updated. We are using the [polkadot.js](https://polkadot.js.org/apps) to test the backend.

## Technical Design Todo

- Implement algorithm to choose route ( We have used randomized routing in the main branch and we are working to optimize shortest route algorithm in the route branch)
- See how we can make use of off-chain workers (OCW)
