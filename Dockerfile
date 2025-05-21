FROM nginx:alpine
WORKDIR /app
COPY backend_startup.sh /
COPY dist/ /usr/share/nginx/html
COPY dir-vigil /usr/local/bin/
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
CMD sh -c "nginx -g 'daemon off;' & /backend_startup.sh"