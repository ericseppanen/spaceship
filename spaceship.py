import os
import pygame
from pygame.locals import *
from pygame.compat import geterror

if not pygame.font:
    print("Warning, fonts disabled")

main_dir = os.path.split(os.path.abspath(__file__))[0]
data_dir = os.path.join(main_dir, "data")


# functions to create our resources
def load_image(name, colorkey=None):
    fullname = os.path.join(data_dir, name)
    try:
        image = pygame.image.load(fullname)
    except pygame.error:
        print("Cannot load image:", fullname)
        raise SystemExit(str(geterror()))
    image = image.convert()
    if colorkey is not None:
        if colorkey == -1:
            colorkey = image.get_at((0, 0))
        image.set_colorkey(colorkey, RLEACCEL)

    image_rect = image.get_rect()
    return image, image_rect


class Spaceship(pygame.sprite.Sprite):
    def __init__(self):
        pygame.sprite.Sprite.__init__(self)  # call Sprite intializer
        self.image, self.rect = load_image("red_ship.png", -1)
        screen = pygame.display.get_surface()
        self.area = screen.get_rect()
        self.rect.topleft = 230, 460

    def update(self):
        pass

    def fly(self, vector):
        """ vector is a 2-tuple like (0, 10) indicating direction """
        newpos = self.rect.move(vector)
        if self.area.contains(newpos):
            self.rect = newpos


def main():
    """this function is called when the program starts.
       it initializes everything it needs, then runs in
       a loop until the function returns."""
    # Initialize Everything
    pygame.init()
    screen = pygame.display.set_mode((500, 500))
    pygame.display.set_caption("Spaceship!")
    pygame.mouse.set_visible(0)

    # Create The Backgound
    background = pygame.Surface(screen.get_size())
    background = background.convert()
    background.fill((0, 0, 0))

    # Display The Background
    screen.blit(background, (0, 0))
    pygame.display.flip()

    # Prepare Game Objects
    clock = pygame.time.Clock()
    spaceship = Spaceship()
    allsprites = pygame.sprite.RenderPlain((spaceship))

    # Remember all the keys that are pressed.
    active_keys = []

    # Main Loop
    going = True
    while going:
        clock.tick(60)

        # Handle Input Events
        for event in pygame.event.get():
            if event.type == QUIT:
                going = False
            elif event.type == KEYDOWN:
                active_keys.append(event.key)
                if event.key == K_ESCAPE:
                    going = False
            elif event.type == KEYUP:
                active_keys.remove(event.key)
            elif event.type == MOUSEBUTTONDOWN:
                pass
            elif event.type == MOUSEBUTTONUP:
                pass

        # Check the list of held keys to compute which way
        # the spaceship should fly.
        fly_direction = None
        for key in active_keys:
            if key == K_RIGHT:
                fly_direction = (1, 0)
            if key == K_LEFT:
                fly_direction = (-1, 0)
        if fly_direction is not None:
            spaceship.fly(fly_direction)

        allsprites.update()

        # Draw Everything
        screen.blit(background, (0, 0))
        allsprites.draw(screen)
        pygame.display.flip()

    pygame.quit()


# Game Over


# this calls the 'main' function when this script is executed
if __name__ == "__main__":
    main()
