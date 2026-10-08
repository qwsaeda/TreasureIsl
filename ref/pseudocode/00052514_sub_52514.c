/* ea:   0x00052514 */
/* name: sub_52514 */
/* size: 484 */
int __fastcall sub_52514(int a1, void *a2)
{
  unsigned int i; // r4
  char v6; // [sp+4h] [bp-20h] BYREF
  bool Buffer[3]; // [sp+5h] [bp-1Fh] BYREF
  DWORD NumberOfBytesWritten; // [sp+8h] [bp-1Ch] BYREF

  if ( a2 == (void *)-1 )
    return 0;
  sub_52E50(a1 + 136);
  Buffer[0] = *(_DWORD *)(a1 + 40) == *(_DWORD *)(a1 + 48);
  WriteFile(a2, Buffer, 1u, &NumberOfBytesWritten, nullptr);
  WriteFile(a2, (LPCVOID)(a1 + 204), 1u, &NumberOfBytesWritten, nullptr);
  WriteFile(a2, (LPCVOID)(a1 + 132), 4u, &NumberOfBytesWritten, nullptr);
  WriteFile(a2, (LPCVOID)(a1 + 64), 4u, &NumberOfBytesWritten, nullptr);
  WriteFile(a2, (LPCVOID)(a1 + 68), 4u, &NumberOfBytesWritten, nullptr);
  WriteFile(a2, (LPCVOID)(a1 + 96), 4u, &NumberOfBytesWritten, nullptr);
  WriteFile(a2, (LPCVOID)(a1 + 100), 4u, &NumberOfBytesWritten, nullptr);
  WriteFile(a2, (LPCVOID)(a1 + 104), 4u, &NumberOfBytesWritten, nullptr);
  WriteFile(a2, (LPCVOID)(a1 + 200), 4u, &NumberOfBytesWritten, nullptr);
  WriteFile(a2, (LPCVOID)(a1 + 124), 4u, &NumberOfBytesWritten, nullptr);
  WriteFile(a2, (LPCVOID)(a1 + 128), 1u, &NumberOfBytesWritten, nullptr);
  v6 = *(_BYTE *)(a1 + 129);
  WriteFile(a2, &v6, 1u, &NumberOfBytesWritten, nullptr);
  v6 = *(_BYTE *)(a1 + 130);
  WriteFile(a2, &v6, 1u, &NumberOfBytesWritten, nullptr);
  if ( *(int *)(a1 + 8) <= 4 )
  {
    for ( i = 0; i < 0xA; ++i )
    {
      v6 = *(_BYTE *)(*(_DWORD *)(a1 + 188) + i);
      WriteFile(a2, &v6, 1u, &NumberOfBytesWritten, nullptr);
    }
  }
  return 1;
}

