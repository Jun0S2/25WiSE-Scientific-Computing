import io
from PyPDF2 import PdfReader
from urllib.request import urlopen

target_url = "https://www.lernhelfer.de/sites/default/files/lexicon/pdf/BWS-DEU2-0947-01.pdf"


response = urlopen(target_url)
pdf_data = response.read()

# Create a PDF reader from memory
reader = PdfReader(io.BytesIO(pdf_data))

# Extract all text
text = ""
for page in reader.pages:
    text += page.extract_text() or ""

# Count words
words = text.split()
print(f"Successfully read {len(words)} words from PDF.")


# possible ans
# curl link | pdftotext - -| tr -s '[...specialchara' \n|sort|uniq -c | sort -nr | wc -l]...