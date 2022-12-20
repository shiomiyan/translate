curl -s 'https://api.deepl.com/v2/glossaries' -H "Authorization: DeepL-Auth-Key $DEEPL_API_KEY" | jq -r '.glossaries[].glossary_id' | while read line
do
  curl -sX DELETE "https://api.deepl.com/v2/glossaries/$line" \
	-H "Authorization: DeepL-Auth-Key $DEEPL_API_KEY"
done