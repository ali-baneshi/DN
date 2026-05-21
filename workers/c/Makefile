CC = gcc
CFLAGS = -Wall -Wextra -std=c11 -O2 -I.
LDFLAGS = -pthread

TARGET = dn-worker-c
SRCS = main.c rules.c
OBJS = $(SRCS:.c=.o)

all: $(TARGET)

$(TARGET): $(OBJS)
	$(CC) $(CFLAGS) -o $@ $^ $(LDFLAGS)

%.o: %.c
	$(CC) $(CFLAGS) -c $< -o $@

clean:
	rm -f $(OBJS) $(TARGET)

.PHONY: all clean
