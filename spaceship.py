import os
import pygame
from pygame.locals import *
from pygame.compat import geterror

if not pygame.font:
    print("Warning, fonts disabled")

main_dir = os.path.split(os.path.abspath(__file__))[0]
data_dir = os.path.join(main_dir, "data")

def load_image(filename):
    fullname = os.path.join(data_dir, filename)
    image = pygame.image.load(fullname)
    return image

class ShipBase(pygame.sprite.Sprite):
    def __init__(self):
        super().__init__()

    def set_image(self, image, center):
        self.image = image
        self.mask = pygame.mask.from_surface(self.image)
        self.rect = image.get_rect()
        self.rect.center = center

    def die(self, image):
        center = self.rect.center
        self.set_image(image, center)

class Spaceship(ShipBase):
    def __init__(self, image):
        super().__init__()
        self.set_image(image, (250, 475))
        screen = pygame.display.get_surface()
        self.area = screen.get_rect()

    def update(self):
        pass

    def fly(self, vector):
        """ vector is a 2-tuple like (0, 10) indicating direction """
        newpos = self.rect.move(vector)
        if self.area.contains(newpos):
            self.rect = newpos

class Enemy(ShipBase):
    def __init__(self, image):
        super().__init__()
        self.set_image(image, (250, 20))
        screen = pygame.display.get_surface()
        self.area = screen.get_rect()
        self.direction = (2, 2)

    def reverse_x(self):
        x, y = self.direction
        self.direction = (-x, y)

    def reverse_y(self):
        x, y = self.direction
        self.direction = (x, -y)

    def update(self):
        # Move around randomly without input
        pos = self.rect.move(self.direction)
        if self.area.contains(pos):
            self.rect = pos
            return
        # Fix the X position
        if pos.right > self.area.right:
            self.reverse_x()
            pos.right = pos.right - 2 * (pos.right - self.area.right)
        if pos.left < self.area.left:
            self.reverse_x()
            pos.left = pos.left - 2 * (pos.left - self.area.left)
        # Fix the Y position
        if pos.bottom > self.area.bottom:
            self.reverse_y()
            pos.bottom = pos.bottom - 2 * (pos.bottom - self.area.bottom)
        if pos.top < self.area.top:
            self.reverse_y()
            pos.top = pos.top - 2 * (pos.top - self.area.top)
        self.rect = pos


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

    # Load images
    red_image = load_image("red_ship.png")
    green_image = load_image("green_ship.png")
    explosion = load_image("explosion.png")

    # Prepare Game Objects
    clock = pygame.time.Clock()
    spaceship = Spaceship(red_image)
    enemy = Enemy(green_image)
    allsprites = pygame.sprite.RenderPlain((spaceship, enemy))

    # Remember all the keys that are pressed.
    active_keys = []

    # Main Loop
    going = True
    exit_time = None
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

        if exit_time:
            current_time = pygame.time.get_ticks()
            if current_time > exit_time:
                going = False
            # Don't do any more screen updates or ship movement
            continue

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
        if pygame.sprite.collide_mask(spaceship, enemy):
            spaceship.die(explosion)
            enemy.die(explosion)
            exit_time = pygame.time.get_ticks() + 1000 # milliseconds

        # Draw Everything
        screen.blit(background, (0, 0))
        allsprites.draw(screen)
        pygame.display.flip()

    pygame.quit()


# Game Over


# this calls the 'main' function when this script is executed
if __name__ == "__main__":
    main()
