"""Original LightDo icon; deterministic, no upstream assets."""
from pathlib import Path
from PIL import Image, ImageDraw
root = Path(__file__).resolve().parent.parent
scale = 4
image = Image.new('RGBA', (256*scale, 256*scale))
mask = Image.new('L', image.size)
ImageDraw.Draw(mask).rounded_rectangle((12*scale,12*scale,244*scale,244*scale),62*scale,fill=255)
for y in range(image.height):
    for x in range(image.width):
        t = (x+y)/(2*(image.width-1))
        rgb = tuple(round(a+(b-a)*t) for a,b in zip((119,219,245),(55,117,232)))
        image.putpixel((x,y),(*rgb,mask.getpixel((x,y))))
draw=ImageDraw.Draw(image)
def line(points,width):
    pts=[(x*scale,y*scale) for x,y in points]
    draw.line(pts,fill='white',width=width*scale,joint='curve')
    for x,y in pts:
        r=width*scale/2
        draw.ellipse((x-r,y-r,x+r,y+r),fill='white')
line([(72,68),(72,187),(184,187)],19)
line([(111,113),(136,138),(186,86)],17)
image=image.resize((256,256),Image.Resampling.LANCZOS)
image.save(root/'assets/lightdo.png')
image.save(root/'assets/lightdo.ico',sizes=[(16,16),(24,24),(32,32),(48,48),(64,64),(128,128),(256,256)])
