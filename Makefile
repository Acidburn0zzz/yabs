CC	= gcc
CXX 	= g++
CFLAGS 	= -pipe -march=x86-64 -mtune=generic -O2 -pipe -fprofile-arcs -fstack-protector --param=ssp-buffer-size=4 -Wall
CXXFLAGS= -pipe -std=c++11 -march=x86-64 -mtune=generic -O2 -pipe -fprofile-arcs -fstack-protector -ftest-coverage --param=ssp-buffer-size=4 -Wall
LINK 	= g++
LFLAGS 	= -Wl,-O1,--sort-common,--as-needed,-z,relro -Wl,-O1
LIBS 	= -lssh2 -lyaml -lgcov -larchive -lgit2
INCPATH = -I/usr/include -Iinclude
DEL	= rm -f
DEL_R	= rm -r
SRC 	= src/dist.cpp \
	  src/env.cpp \
	  src/gen.cpp \
	  src/interface.cpp \
	  src/parser.cpp \
	  src/profile.cpp \
	  src/remote.cpp \
	  src/yabs.cpp
OBJ 	= dist.o \
	  env.o \
	  gen.o \
	  interface.o \
	  parser.o \
	  profile.o \
	  remote.o \
	  yabs.o
TRGT 	= yabs

first: all

.SUFFIXES: .o .c .cpp .cc .cxx .C

.cpp.o:
	$(CXX) -c $(CXXFLAGS) $(INCPATH) -o "$@" "$<"

.cc.o:
	$(CXX) -c $(CXXFLAGS) $(INCPATH) -o "$@" "$<"

.cxx.o:
	$(CXX) -c $(CXXFLAGS) $(INCPATH) -o "$@" "$<"

.C.o:
	$(CXX) -c $(CXXFLAGS) $(INCPATH) -o "$@" "$<"

.c.o:
	$(CC) -c $(CFLAGS) $(INCPATH) -o "$@" "$<"

all: $(TRGT)

$(TRGT): $(OBJ)
	$(CXX) $(LFLAGS) -o $(TRGT) $(OBJ) $(LIBS)

dist.o: src/dist.cpp
	$(CXX) -c $(CXXFLAGS) $(INCPATH) -o dist.o src/dist.cpp

env.o: src/env.cpp
	$(CXX) -c $(CXXFLAGS) $(INCPATH) -o env.o src/env.cpp

gen.o: src/gen.cpp
	$(CXX) -c $(CXXFLAGS) $(INCPATH) -o gen.o src/gen.cpp

interface.o: src/interface.cpp
	$(CXX) -c $(CXXFLAGS) $(INCPATH) -o interface.o src/interface.cpp

parser.o: src/parser.cpp
	$(CXX) -c $(CXXFLAGS) $(INCPATH) -o parser.o src/parser.cpp

profile.o: src/profile.cpp
	$(CXX) -c $(CXXFLAGS) $(INCPATH) -o profile.o src/profile.cpp

remote.o: src/remote.cpp
	$(CXX) -c $(CXXFLAGS) $(INCPATH) -o remote.o src/remote.cpp

yabs.o: src/yabs.cpp
	$(CXX) -c $(CXXFLAGS) $(INCPATH) -o yabs.o src/yabs.cpp

clean:
	$(DEL) $(OBJ)
	$(DEL) yabs
	$(DEL) *~ core *.core
	$(DEL) *.gcno *.gcda *.gcov
