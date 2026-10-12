/* ea:   0x000526F8 */
/* name: sub_526F8 */
/* size: 532 */
int __fastcall sub_526F8(_DWORD *a1, void *a2)
{
  int v4; // r1
  int v5; // r0
  int v6; // r0
  int v7; // r0
  unsigned int i; // r4
  char v10; // [sp+4h] [bp-1Ch] BYREF
  _BYTE Buffer[3]; // [sp+5h] [bp-1Bh] BYREF
  DWORD NumberOfBytesRead; // [sp+8h] [bp-18h] BYREF

  if ( a2 == (void *)-1 )
    return 0;
  sub_53138(a1 + 34);
  ReadFile(a2, Buffer, 1u, &NumberOfBytesRead, nullptr);
  if ( Buffer[0] != 0 )
  {
    v4 = a1[26];
    a1[10] = a1[12];
    v5 = a1[22];
    a1[24] = v4;
    a1[18] = v5;
    v6 = a1[9];
    a1[19] = a1[23];
    a1[27] = 0;
    a1[7] = v6;
  }
  ReadFile(a2, a1 + 51, 1u, &NumberOfBytesRead, nullptr);
  ReadFile(a2, a1 + 33, 4u, &NumberOfBytesRead, nullptr);
  ReadFile(a2, a1 + 16, 4u, &NumberOfBytesRead, nullptr);
  ReadFile(a2, a1 + 17, 4u, &NumberOfBytesRead, nullptr);
  a1[3] = sub_53808(a1[6] + 52, a1[16], a1[17]);
  ReadFile(a2, a1 + 24, 4u, &NumberOfBytesRead, nullptr);
  ReadFile(a2, a1 + 25, 4u, &NumberOfBytesRead, nullptr);
  ReadFile(a2, a1 + 26, 4u, &NumberOfBytesRead, nullptr);
  ReadFile(a2, a1 + 50, 4u, &NumberOfBytesRead, nullptr);
  ReadFile(a2, a1 + 31, 4u, &NumberOfBytesRead, nullptr);
  ReadFile(a2, a1 + 32, 1u, &NumberOfBytesRead, nullptr);
  ReadFile(a2, &v10, 1u, &NumberOfBytesRead, nullptr);
  *((_BYTE *)a1 + 129) = v10;
  ReadFile(a2, &v10, 1u, &NumberOfBytesRead, nullptr);
  v7 = a1[2];
  *((_BYTE *)a1 + 130) = v10;
  if ( v7 <= 4 )
  {
    for ( i = 0; i < 0xA; ++i )
    {
      ReadFile(a2, &v10, 1u, &NumberOfBytesRead, nullptr);
      *(_BYTE *)(a1[47] + i) = v10;
    }
  }
  return 1;
}

