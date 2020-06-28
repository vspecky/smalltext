# SmallText
Smalltext is a command-line tool for compressing/decompressing text files written in Rust. It does so by using a technique known as Huffman Coding.  
A project to help me learn bout Rust's File options.

## What is Huffman Coding?
The basic gist of it is as follows.  
Lets say you have a text file with the string "bbbba". Each character of the file occupies exactly one byte of space (Non-ascii UTF-8 characters occupy more but they still can be divided into single byte units) so the total size of the file is 5 bytes. But we don't really need 8 bytes for encoding a single character since there are only two characters in total in the file. Lets say we encode all the b's as 1 and a's as 0. In that case we can encode our original string as 11110 which, when converted into a byte, would be 00011110, hence we compressed 5 bytes of data down into a single byte.  
Now the question arises, wouldn't this cause errors in the actual byte representation? We have 3 leading zeros before the actual string so wouldn't it ultimately decompress to 'aaabbbba'? You would be quite right, and this is why we use Huffman Coding.  
The Huffman Coding algorithm ensures that all characters receive a unique encoding that won't cause sequential intersections with encodings of any other character. Additionally HC prefers to give shorter codes to more frequently appearing characters and longer ones to less frequently appearing characters, ultimately leading to compression. Read more [here](https://en.wikipedia.org/wiki/Huffman_coding)  

## How does SmallText work?
### Compression 
- Given any text file for compression, the Encoder derives a Huffman Coding table with encodings for all characters in the file.  
- A new file with a '.cmpr' extension is created. This will be the compressed version of the original file.  
- Before encoding the original text file, the Encoder serializes the HC table in a way such that enough information is provided for the Decoder to deserialize the table and appends it to the start of the file.  
- Now the Encoder starts encoding the original text using the HC table and simultaneously writing the encoded data to the .cmpr file.  
- Writing to the output file can only be done in the form of bytes, so the Encoder converts the encoded data into continuous bytes such that each byte may contain multiple characters worth of information and if encoding for a character doesn't fit at the end of a byte, the remaining part is carried over to the start of the next byte and so on till the end of the file.

### Decompression
- Given any .cmpr file for decompression, the Decoder deserializes and retrieves the HC table from the start of the file.  
- Once it has the table, the Decoder just scans the .cmpr file byte by byte and decodes the encoded information using the table, writing it to a file with a name identical to the .cmpr file, just without the .cmpr extension.  

## Usage
```
smalltext <flag> <path/to/file>
```
Where the `<flag>` argument can accept one of two flags :- 
- Compression: `-c`  
- Decompression `-d`