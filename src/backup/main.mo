import Array "mo:base/Array";
import Nat64 "mo:base/Nat64";
actor {
    stable var data : [[Nat8]] = [];

    public func addData(chunk : [Nat8]) : async () {
        data := Array.flatten<[Nat8]>([data, [chunk]]);
    };

    public query func getChunk(index : Nat64) : async [Nat8] {
        data[Nat64.toNat(index)];
    };

    public query func size() : async Nat64 { Nat64.fromNat(data.size()) };
    public func clear() : async () { data := [] };
};
