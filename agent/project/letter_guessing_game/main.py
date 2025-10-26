import random  #[Alex]

def play_game():  #[Alex]
    letters = 'abcdefghijklmnopqrstuvwxyz'  #[Alex]
    secret_letter = random.choice(letters)  #[Alex]
    guess = ''  #[Alex]
    attempts = 0  #[Alex]

    while guess != secret_letter:  #[Alex]
        try:  #[Alex]
            guess = input('Guess a letter: ')  #[Alex]
        except EOFError:  #[Alex]
            print('Input ended. Exiting game.')  #[Alex]
            break  #[Alex]
        attempts += 1  #[Alex]
    print(f'Congratulations! You guessed the letter {secret_letter} in {attempts} attempts.')  #[Alex]

if __name__ == '__main__':  #[Alex]
    play_game()  #[Alex]
