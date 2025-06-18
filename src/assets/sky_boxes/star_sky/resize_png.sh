#!/bin/bash


# Проходим по всем PNG-файлам в текущей папке
for file in *.png; do
    if [ -f "$file" ]; then
        # Получаем текущие размеры изображения
        dimensions=$(identify -format "%wx%h" "$file")
        width=$(echo "$dimensions" | cut -d'x' -f1)
        height=$(echo "$dimensions" | cut -d'x' -f2)
        
        # Вычисляем новые размеры (в 2 раза меньше)
        new_width=$((width / 2))
        new_height=$((height / 2))
        
        # Изменяем размер (создаём копию с префиксом resized_)
        convert "$file" -resize "${new_width}x${new_height}" "$file"
        
        echo "Изменён: $file (${width}x${height} -> ${new_width}x${new_height})"
    fi
done

echo "Готово!"
