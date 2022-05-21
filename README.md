# Asset Tracking System by Substrate

## Shipment Tracking and Verification on Substrate_

The aim of this project is to build a shipment tracking and verification blockchain using substrate. The “hello world” example will have the functionality to create a shipment with a unique ID at predefined origin points. To keep things simple the route will be defined only at the init stage and will be assumed to be static for the purpose of the demo. The verification component consists of generating a secret key and a temporary owner for every shipment at every transit point. The key and the owner will be updated at every transit point.   

For example, let us say that there are N transit points {Y1,Y2,...,YN}
A shipment originates at Y4 and is destined for Y11. The route is set as {Y4,Y8,Y22,Y11}, the package is assigned a UID of , say, 112 and a secret key as well as the owner (the next node Y8) is set. The package will also have a physical tag and seal that can be scanned to get the key. On receiving the package, Y8 calls an update function that accepts the package UID and scanned key. It verifies that the key and the owner information is correct. On verification it generates a new key and the new owner (Y22). At the destination the package is marked as Shipped and the keys/owners are set to None.

> Blockchain Overview

Players

Manager : One account that manages a number of transit points and has the power to add or remove transit points. Alternatives to a centralized entity ?

Transit Points : N transit points. Each one has a unique id and serves as both a sender and a receiver. In the simplest case, each originating transit point also sets the transit route and calls a blockchain function that creates a secret key and the identity of the node set to receive the package on the route chart. The secret key is also embedded in the physical package for the receiver and only the correct receiver can decode the key in order to keep the package moving along the shipment chain. The key generation process repeats at every step.

Customers : In the simplest implementation case, customers can only access the state of the chain and the current location/status of the package. 


> Methods

Administrative : Manage transit points. Add() and Remove() transit nodes in the base case.

Core : Functions to create the shipment, generate keys , update keys and package status at every transit point

> Technical Design
_Todo_
- Add secret key functionality
- Implement algorithm to choose route
- See how we can make use of off-chain workers (OCW)
