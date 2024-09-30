# What does it do?
It takes a gerber file* and returns a dxf file*
# Why the asterisks?
I tested this on fusion 360 (2024?) and while it can take an exported layer from Fusion and make a valid DXF file, Fusion refuses to read it properly. converting to dwg online and then opening it makes it work right. (although that process seems to change the scale by a factor of 10)
![image](https://github.com/user-attachments/assets/add1f868-eaf0-41a4-9c41-51382dea5b55)
![image](https://github.com/user-attachments/assets/43c8469f-ccb3-46ac-9e07-e8c3eb15ac68)
![image](https://github.com/user-attachments/assets/f1f38ab8-fb6e-4bfe-9c09-259aa8702509)

# Printed stencil (results)
Here is the dxf after importing to fusion, extruding it, and then adding a lip to place the PCB. The stencil was printed on a Bambu X1C with 0.2mm nozzle, 0.06mm layer height. Ended up being 0.18mm thick, where the PCBWay default is 0.12mm thick for their stencils.
I haven't used it because I'm still waiting on the PCB, but heres the pros and cons;
## Pros:
- Faster (30 minutes vs 5 days + shipping)
- Cheaper (cents of filament as opposed to ~$50)
- Can add custom locating features, including a lip in the 3rd dimension
## Cons:
- Lower quality (I'll attach results when I can)
- slightly thicker (0.12mm seems to be doable though which is the PCBWay stencil default)
- You have to debug and fix my code if; you aren't using fusion360, your gerber files uses rectangular interpolated apertures, your gerber files actually use aperture macros, they use arc interpolation, or anything else I haven't thought of.

![20240929_190759](https://github.com/user-attachments/assets/0efde98c-5edd-426c-a91d-155007a332e0)
![20240929_190736](https://github.com/user-attachments/assets/4fd37eda-882e-4e6f-a21c-01c3e79113ea)


# Basically it works on my machine, email me if you really want to work on this, but I got what I needed out of it
There are dirty tricks everywhere because Gerber is the worst. Basically G-Code but at home.

# Error handling
Honestly non-existant. I would want to work more on the parser lib first to make it not full of panics.

# Why did I do it?
PCBWay (not sponsored) charged me 3 times more for adding a single stencil to my order. This project however only cost me days of work and about 12 beers.
