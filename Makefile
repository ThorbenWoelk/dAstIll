.PHONY: start stop restart

start:
	./start_app.sh

stop:
	./end_app.sh

restart:
	./end_app.sh
	./start_app.sh
