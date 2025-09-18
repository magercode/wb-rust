FLAG = g++
CFLAGS = -g -Wall -Wextra -Werror -std=c++17
INCLUDE = -Isrc/*.hpp
SRC = $(wildcard src/*.cpp)

build:
	@echo "membangun wibu..."
	@sleep 2
	@echo "Sumber: $(SRC)"
	@$(FLAG) --version
	@echo "Using flags: $(CFLAGS)"
	@echo "Including headers: $(INCLUDE)"
	@echo "Output binary: lari ada wibu"
	$(FLAG) $(CFLAGS) $(INCLUDE) $(SRC) -o wibu

clean:
	rm -f wibu