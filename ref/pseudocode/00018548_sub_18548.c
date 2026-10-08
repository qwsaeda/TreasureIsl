/* ea:   0x00018548 */
/* name: sub_18548 */
/* size: 368 */
int *__fastcall sub_18548(int *a1, _WORD *a2, int a3)
{
  unsigned int v6; // r4
  unsigned int v7; // r5
  int *v8; // r0
  int v9; // r6
  _WORD *v10; // r5
  _WORD *v11; // r0
  _WORD *v12; // r4
  _BYTE v14[3]; // [sp+5h] [bp-27h] BYREF
  int v15; // [sp+8h] [bp-24h] BYREF
  unsigned int v16; // [sp+Ch] [bp-20h] BYREF

  if ( a2 == (_WORD *)a3 )
    return a1;
  v6 = (a1[1] - *a1) >> 1;
  v7 = (a3 - (int)a2) >> 1;
  v16 = v6;
  if ( v7 > 0x7FFFFFFE || v6 > 2147483646 - v7 )
    TerminateProcess((HANDLE)0x42, 0);
  if ( v7 + v6 <= ((a1[2] - *a1) >> 1) - 1 )
  {
    sub_186B8(a2 + 1, a3, a1[1] + 2, v14, 0);
    *(_WORD *)(a1[1] + 2 * v7) = 0;
    *(_WORD *)a1[1] = *a2;
    a1[1] += 2 * v7;
    return a1;
  }
  v15 = (a3 - (int)a2) >> 1;
  v8 = &v15;
  if ( v6 >= v7 )
    v8 = (int *)&v16;
  v9 = *v8 + v6 + 1;
  v10 = sub_1343C((int)(a1 + 2), v9);
  v11 = sub_13898(*a1, a1[1], v10);
  v12 = sub_13898((int)a2, a3, v11);
  *v12 = 0;
  if ( *a1 != 0 )
    free((void *)*a1);
  *a1 = (int)v10;
  a1[1] = (int)v12;
  a1[2] = (int)&v10[v9];
  return a1;
}

