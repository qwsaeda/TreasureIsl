/* ea:   0x00056F64 */
/* name: sub_56F64 */
/* size: 256 */
int __fastcall sub_56F64(_DWORD *a1, void *a2)
{
  int v4; // r1
  int v6; // r3
  int v7; // r4
  int v8; // r2
  int v9; // r1
  DWORD NumberOfBytesRead; // [sp+4h] [bp-18h] BYREF

  if ( a2 == (void *)-1 )
    return 0;
  sub_526F8(a1, a2);
  ReadFile(a2, a1 + 52, 4u, &NumberOfBytesRead, nullptr);
  ReadFile(a2, a1 + 53, 2u, &NumberOfBytesRead, nullptr);
  ReadFile(a2, a1 + 54, 4u, &NumberOfBytesRead, nullptr);
  v4 = sub_39948(a1[6] + 520, a1 + 54)[5];
  a1[4] = v4;
  if ( v4 == 0 )
    return 0;
  v6 = a1[6];
  v7 = a1[54];
  v8 = *(_DWORD *)(*(_DWORD *)(v6 + 32) + 4 * v7 + 60);
  a1[15] = v8;
  if ( v8 == 0 )
    return 0;
  a1[8] = 32 * *(_DWORD *)(*(_DWORD *)(v6 + 264) + 4 * v7 - 4);
  v9 = 32 * *(_DWORD *)(*(_DWORD *)(v6 + 264) + 4 * v7 - 4);
  a1[7] = a1[8];
  a1[9] = v9;
  return 1;
}

