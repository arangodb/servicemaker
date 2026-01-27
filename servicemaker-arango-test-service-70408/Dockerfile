FROM arangodb/py13base:latest

USER root

COPY ./scripts /scripts
COPY arango-test-service /project/arango-test-service
RUN chown -R user:user /project/arango-test-service

USER user
WORKDIR /project/arango-test-service

RUN /scripts/prepareproject.sh
RUN echo "/project/arango-test-service/main.py" > /project/entrypoint

EXPOSE 8000

CMD ["/bin/bash", "-c", ". /home/user/.local/bin/env && . /home/user/the_venv/bin/activate && export PYTHONPATH=/project/the_venv/lib/python3.13/site-packages && cd /project/arango-test-service && python main.py"]
