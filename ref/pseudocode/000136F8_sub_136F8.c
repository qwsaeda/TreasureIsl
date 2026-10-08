/* ea:   0x000136F8 */
/* name: sub_136F8 */
/* size: 344 */
_DWORD *__fastcall sub_136F8(_DWORD *a1, char *a2, char *a3)
{
  unsigned int v6; // r5
  unsigned int v7; // r4
  int *v8; // r0
  int v9; // r6
  char *v10; // r5
  void *v11; // r0
  _BYTE *v12; // r0
  char *v13; // r4
  char v15[4]; // [sp+0h] [bp-2Ch] BYREF
  int v16; // [sp+4h] [bp-28h] BYREF
  unsigned int v17; // [sp+8h] [bp-24h] BYREF

  if ( a2 == a3 )
    return a1;
  v6 = (a3 - a2) >> 1;
  v7 = (a1[1] - *a1) >> 1;
  v17 = v7;
  if ( v6 > 0x7FFFFFFE || v7 > 2147483646 - v6 )
    TerminateProcess((HANDLE)0x42, 0);
  if ( v6 + v7 <= ((a1[2] - *a1) >> 1) - 1 )
  {
    if ( a3 != a2 + 2 )
      memmove((void *)(a1[1] + 2), a2 + 2, a3 - (a2 + 2));
    *(_WORD *)(a1[1] + 2 * v6) = 0;
    *(_WORD *)a1[1] = *(_WORD *)a2;
    a1[1] += 2 * v6;
    return a1;
  }
  v16 = (a3 - a2) >> 1;
  v8 = &v16;
  if ( v7 >= v6 )
    v8 = (int *)&v17;
  v9 = *v8 + v7 + 1;
  v10 = (char *)sub_1343C((int)(a1 + 2), v9);
  v11 = (void *)sub_13898(*a1, a1[1], v10, v15);
  v12 = memmove(v11, a2, a3 - a2);
  *(_WORD *)&a3[v12 - a2] = 0;
  v13 = &a3[v12 - a2];
  if ( *a1 != 0 )
    free((void *)*a1);
  *a1 = v10;
  a1[1] = v13;
  a1[2] = &v10[2 * v9];
  return a1;
}

