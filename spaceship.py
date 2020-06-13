#!/usr/bin/python3

import copy
import os
import pygame
from pygame.locals import *
from pygame.compat import geterror
from random import randint, getrandbits

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
            if self.area.contains(newpos):
                self.rect = newpos
            else:
                self.kill()

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
    def __init__(self, image, location, direction):
        super().__init__()
        self.set_image(image, location)
        screen = pygame.display.get_surface()
        self.area = screen.get_rect()
        self.direction = direction

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


# images are globals
red_image = load_image("red_ship.png")
green_image = load_image("green_ship.png")
green_image = pygame.transform.flip(green_image, True, True)
torpedo_image = load_image("torpedo.png")
explosions = [
    load_image("explosion1.png"),
    load_image("explosion2.png"),
    load_image("explosion3.png"),
    load_image("explosion4.png"),
    load_image("explosion5.png"),
    load_image("explosion6.png"),
]

class SpaceshipGame:
    def __init__(self):
        pygame.init()
        self.screen = pygame.display.set_mode((500, 500))
        pygame.display.set_caption("Spaceship!")
        pygame.mouse.set_visible(0)

        # Create The Backgound
        bg = pygame.Surface(self.screen.get_size())
        self.background = bg.convert()
        self.background.fill((0, 0, 0))

        # Display The Background
        self.screen.blit(self.background, (0, 0))
        pygame.display.flip()

        # Prepare Game Objects
        self.exit_time = None
        self.clock = pygame.time.Clock()
        self.current_time = 0
        self.spaceship = Spaceship(red_image)
        self.players = pygame.sprite.GroupSingle(self.spaceship)
        self.enemies = pygame.sprite.RenderPlain()
        self.projectiles = pygame.sprite.RenderPlain()

        # Remember all the keys that are pressed.
        self.active_keys = []

        # Keep score
        self.score = 0

        # Set the time for the first spawned enemy
        self.enemy_spawn_time = 3000
        self.max_enemies = 10

    def handle_events(self):
        for event in pygame.event.get():
            if event.type == QUIT:
                self.exit_time = 0
            elif event.type == KEYDOWN:
                self.active_keys.append(event.key)
                if event.key == K_ESCAPE:
                    self.exit_time = 0
                if event.key == K_SPACE:
                    torp_pos = self.spaceship.rect.midtop
                    torp_dir = (0, -5)
                    torpedo = Projectile(torpedo_image, torp_pos, torp_dir)
                    self.projectiles.add(torpedo)

            elif event.type == KEYUP:
                self.active_keys.remove(event.key)
            elif event.type == MOUSEBUTTONDOWN:
                pass
            elif event.type == MOUSEBUTTONUP:
                pass

    def player_move(self):
        # Check the list of held keys to compute which way
        # the spaceship should fly.
        fly_direction = None
        for key in self.active_keys:
            if key == K_RIGHT:
                fly_direction = (1, 0)
            if key == K_LEFT:
                fly_direction = (-1, 0)
        if fly_direction is not None:
            self.spaceship.fly(fly_direction)

    def spawn_enemies(self):
        if self.current_time > self.enemy_spawn_time:
            if len(self.enemies.sprites()) < self.max_enemies:
                location = (randint(40, 460), 20)
                x_direction = -2 + 4 * getrandbits(1)
                direction = (x_direction, 2)
                new_enemy = Enemy(green_image, location, direction)
                self.enemies.add(new_enemy)
                self.enemy_spawn_time += max(1000, 5000 - self.score * 5)

    def sprite_updates(self):
        self.players.update()
        self.enemies.update()
        self.projectiles.update()
        collide_dict = pygame.sprite.groupcollide(
                self.enemies,
                self.projectiles,
                False,
                True,
                pygame.sprite.collide_mask)
        for hit_enemy in collide_dict:
            if not hit_enemy.dying:
                self.score += 100
                print('Enemy destroyed! Score={}'.format(self.score))
                hit_enemy.die(Animation(explosions, 10))
        collide_dict = pygame.sprite.groupcollide(
                self.players,
                self.enemies,
                False,
                False,
                pygame.sprite.collide_mask)
        for hit_player, hit_enemies in collide_dict.items():
            if not hit_player.dying:
                hit_player.die(Animation(explosions, 10))
            for hit_enemy in hit_enemies:
                if not hit_enemy.dying:
                    hit_enemy.die(Animation(explosions, 10))
            self.exit_time = pygame.time.get_ticks() + 1000 # milliseconds

    def draw(self):
        self.screen.blit(self.background, (0, 0))
        self.players.draw(self.screen)
        self.enemies.draw(self.screen)
        self.projectiles.draw(self.screen)
        pygame.display.flip()

    def run(self):
        while True:
            self.clock.tick(60)
            self.current_time = pygame.time.get_ticks()
            if self.exit_time is not None:
                if self.current_time > self.exit_time:
                    break
            self.handle_events()
            self.spawn_enemies()
            self.player_move()
            self.sprite_updates()
            self.draw()

        # When we exit the loop the game is over.
        pygame.quit()


def main():
    SpaceshipGame().run()


# this calls the 'main' function when this script is executed
if __name__ == "__main__":
    main()
