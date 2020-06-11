#!/usr/bin/python3

import copy
import os
import pygame
from pygame.locals import *
from pygame.compat import geterror

main_dir = os.path.split(os.path.abspath(__file__))[0]
data_dir = os.path.join(main_dir, "data")

def load_image(filename):
    fullname = os.path.join(data_dir, filename)
    image = pygame.image.load(fullname)
    return image

class Animation:
    def __init__(self, image_list, delay):
        self.image_list = copy.copy(image_list)
        self.delay = delay
        self.countdown = 0

    def __bool__(self):
        """ Return True if the animation is ongoing """
        return bool(self.image_list)

    def next(self):
        """ Return a new image if it's time, None otherwise """
        self.countdown -= 1
        if self.countdown > 0:
            return None
        # Time for a new image, maybe?
        try:
            image = self.image_list.pop(0)
            self.countdown = self.delay
            return image
        except IndexError:
            return None

class SpriteBase(pygame.sprite.Sprite):
    def __init__(self):
        super().__init__()
        self.animation = None
        self.dying = False

    def set_image(self, image, center):
        self.image = image
        self.mask = pygame.mask.from_surface(self.image)
        self.rect = image.get_rect()
        self.rect.center = center

    def update(self):
        if self.animation:
            image = self.animation.next()
            if image:
                center = self.rect.center
                self.set_image(image, center)
        else:
            if self.dying:
                # We are dying and have already finished our animation.
                self.kill()

    def die(self, animation):
        self.dying = True
        self.animation = animation

class Projectile(SpriteBase):
    def __init__(self, image, location, direction):
        super().__init__()
        self.set_image(image, location)
        screen = pygame.display.get_surface()
        self.area = screen.get_rect()
        self.direction = direction
        self.active = True

    def update(self):
        if self.active:
            newpos = self.rect.move(self.direction)
            if not self.area.contains(newpos):
                self.deactivate()

    def deactivate(self):
        self.active = False
        self.rect = None

class Spaceship(SpriteBase):
    def __init__(self, image):
        super().__init__()
        self.set_image(image, (250, 475))
        screen = pygame.display.get_surface()
        self.area = screen.get_rect()

    def update(self):
        super().update()

    def fly(self, vector):
        """ vector is a 2-tuple like (0, 10) indicating direction """
        newpos = self.rect.move(vector)
        if self.area.contains(newpos):
            self.rect = newpos

class Enemy(SpriteBase):
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
        super().update()

        if self.dying:
            return

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
    explosions = [
        load_image("explosion1.png"),
        load_image("explosion2.png"),
        load_image("explosion3.png"),
        load_image("explosion4.png"),
        load_image("explosion5.png"),
        load_image("explosion6.png"),
    ]

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
        else:
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
            if not spaceship.dying:
                spaceship.die(Animation(explosions, 10))
            if not enemy.dying:
                enemy.die(Animation(explosions, 10))
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
