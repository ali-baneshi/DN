CC = gcc
CFLAGS = -Wall -Wextra -std=c11 -O2 -I.
<<<<<<< HEAD
LDFLAGS = -pthread
=======
>>>>>>> feature/persistent-workers

TARGET = dn-worker-c
SRCS = main.c rules.c
OBJS = $(SRCS:.c=.o)

all: $(TARGET)

$(TARGET): $(OBJS)
<<<<<<< HEAD
	$(CC) $(CFLAGS) -o $@ $^ $(LDFLAGS)
=======
	$(CC) $(CFLAGS) -o $@ $^
>>>>>>> feature/persistent-workers

%.o: %.c
	$(CC) $(CFLAGS) -c $< -o $@

clean:
	rm -f $(OBJS) $(TARGET)

.PHONY: all clean
