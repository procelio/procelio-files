# The InventoryFile File Format: Version `2`

All multi-byte numbers are big-endian and unsigned unless otherwise mentioned

### Header Region
```
4 bytes: magic number (0xC50CB115)
4 bytes: version (3)

4 bytes: number of blocks "N"

N times:
  4 bytes: block ID
  4 bytes SIGNED: block count

4 bytes: number of cosmetics "M"
M times:
  4 bytes: cosmetic ID
  4 bytes SIGNED: cosmetic count
```

### JSON
Check src/files/inventory/inventory.rs for the specifics. Input should be of form
```
{
  "parts": [
    {
      "id": 12,
      "name": "tier 1 wheel",
      "count": 6
    },
    {
      "id": 432,
      "name": "idk some gun",
      "count": 3
    } 
  ]
}
```
The "name" field is used for readability and is disregarded for serialization