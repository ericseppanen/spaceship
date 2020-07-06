#!/usr/bin/python3

import copy
import os
import pygame
from pygame.locals import *
from random import randint, getrandbits

main_dir = os.path.split(os.path.abspath(__file__))[0]
data_dir = os.path.join(main_dir, "data")

def load_image(filename):
    fullname = os.path.join(data_dir, filename)
    image = pygame.image.load(fullname)
    return image

class Animation:
    """ A helper class that cycles through a list of images """

    def __init__(self, image_list, delay):
        """ set up the list of images and the delay between each """

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
    """ A base class for our sprites, including commonly used functions """
    def __init__(self):
        super().__init__()
        screen = pygame.display.get_surface()
        self.area = screen.get_rect()
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
    """ A projectile sprite, that just moves in a constant direction """
    def __init__(self, image, location, direction):
        super().__init__()
        self.set_image(image, location)
        self.direction = direction

    def update(self):
        newpos = self.rect.move(self.direction)
        if self.area.contains(newpos):
            self.rect = newpos
        else:
            self.kill()

class Spaceship(SpriteBase):
    """ The player's ship """
    def __init__(self, image):
        super().__init__()
        self.set_image(image, (250, 475))

    def fly(self, vector):
        """ vector is a 2-tuple like (0, 10) indicating direction """
        newpos = self.rect.move(vector)
        if self.area.contains(newpos):
            self.rect = newpos

class Enemy(SpriteBase):
    def __init__(self, image, location, direction, points):
        super().__init__()
        self.set_image(image, location)
        self.direction = direction
        self.points = points

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
blue_image = load_image("blue_ship.png")
blue_image = pygame.transform.flip(blue_image, True, True)

torpedo_image = load_image("torpedo.png")
explosions = [
    load_image("explosion1.png"),
    load_image("explosion2.png"),
    load_image("explosion3.png"),
    load_image("explosion4.png"),
    load_image("explosion5.png"),
    load_image("explosion6.png"),
]

REGULAR_ENEMY_POINTS = 100
FIGHTER_ENEMY_POINTS = 300

GAME_LEVELS = {
    # level 1
    1: {
        'enemy_count': 5,
        'spawn_rate': 10,
        'fighter_count': 1,
    },
    # level 2
    2: {
        'enemy_count': 10,
        'spawn_rate': 20,
        'fighter_count': 2,
    },
    # level 3
    3: {
        'enemy_count': 15,
        'spawn_rate': 30,
        'fighter_count': 3,
    },
}

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
        self.init_fonts()

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

        # Set up the game level and phase
        self.level_num = 0
        self.game_phase = 'starting'

        # Keep score
        self.score = 0
        self.redraw_background()

    def init_fonts(self):
        font_path = os.path.join(data_dir, 'font', 'monoMMM_5.ttf')
        self.score_font = pygame.font.Font(font_path, 18)

    def redraw_background(self):
        score_str = '{:05}'.format(self.score)
        color = (200, 200, 200)
        text = self.score_font.render(score_str, False, color)
        textpos = text.get_rect(centerx=self.background.get_width() / 2)
        self.background.fill((0, 0, 0))
        self.background.blit(text, textpos)
        if self.game_phase == 'getready':
            level_str = 'LEVEL {}'.format(self.level_num)
            center_text = self.score_font.render(level_str, False, color)
            center_text_pos = center_text.get_rect(center=self.background.get_rect().center)
            self.background.blit(center_text, center_text_pos)

    def player_shoot(self):
        if self.game_phase != 'playing':
            return
        torp_pos = self.spaceship.rect.midtop
        torp_dir = (0, -5)
        torpedo = Projectile(torpedo_image, torp_pos, torp_dir)
        self.projectiles.add(torpedo)

    def handle_events(self):
        for event in pygame.event.get():
            if event.type == QUIT:
                self.exit_time = 0
            elif event.type == KEYDOWN:
                self.active_keys.append(event.key)
                if event.key == K_ESCAPE:
                    self.exit_time = 0
                if event.key == K_SPACE:
                    self.player_shoot()

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
        if self.level_enemies == 0:
            return
        if self.current_time > self.enemy_spawn_time:
            self.level_enemies -= 1
            location = (randint(40, 460), 20)
            x_direction = -2 + 4 * getrandbits(1)
            direction = (x_direction, 2)
            new_enemy = Enemy(green_image, location, direction, REGULAR_ENEMY_POINTS)
            self.enemies.add(new_enemy)
            self.enemy_spawn_time += self.spawn_delay

    def spawn_fighters(self):
        if self.level_fighters == 0:
            return

        total = self.level['fighter_count']
        # This is a fraction that tells us when to spawn a fighter
        fighter_threshold = 1.0 - self.level_fighters / (total + 1)

        # This is a fraction of how many enemies we killed
        killed_frac = self.enemies_killed / self.level['enemy_count']

        if killed_frac > fighter_threshold:
            # We should spawn a new fighter now.
            self.level_fighters -= 1
            location = (randint(40, 460), 20)
            x_direction = -3 + 6 * getrandbits(1)
            direction = (x_direction, 0)
            new_enemy = Enemy(blue_image, location, direction, FIGHTER_ENEMY_POINTS)
            self.enemies.add(new_enemy)

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
                self.score += hit_enemy.points
                self.redraw_background()
                hit_enemy.die(Animation(explosions, 10))
                self.enemies_killed += 1
                self.spawn_fighters()
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

    def init_level(self):
        self.enemies_killed = 0
        self.level_num += 1
        self.level = GAME_LEVELS[self.level_num]
        self.level_enemies = self.level['enemy_count']
        self.level_fighters = self.level['fighter_count']
        self.spawn_delay = 20000 // self.level['spawn_rate']

    def phase_check(self):
        # This will trigger the 'getready' phase
        if self.game_phase == 'starting':
            self.next_phase_time = 0

        if self.next_phase_time is None:
            return
        if self.current_time < self.next_phase_time:
            return
        self.next_phase_time = None
        if self.game_phase == 'getready':
            self.game_phase = 'playing'
            self.enemy_spawn_time = self.current_time + 2000
        else:
            self.init_level()
            self.game_phase = 'getready'
            self.next_phase_time = self.current_time + 2000
        self.redraw_background()

    def run(self):
        while True:
            self.clock.tick(60)
            self.current_time = pygame.time.get_ticks()
            if self.exit_time is not None:
                if self.current_time > self.exit_time:
                    break
            self.phase_check()
            self.handle_events()
            if self.game_phase == 'playing':
                self.spawn_enemies()
                # sprites() returns a list, so we're checking if the list is empty
                if self.level_enemies == 0 and self.enemies.sprites():
                    self.next_phase_time = self.current_time + 1000
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
