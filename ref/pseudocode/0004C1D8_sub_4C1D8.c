/* ea:   0x0004C1D8 */
/* name: sub_4C1D8 */
/* size: 220 */
unsigned __int8 *__fastcall sub_4C1D8(_DWORD *a1, int a2)
{
  unsigned __int8 *v4; // r4
  unsigned __int8 *result; // r0
  int v6; // r5
  char v7; // t1
  int v8; // [sp+0h] [bp-40h]
  _DWORD v9[8]; // [sp+8h] [bp-38h] BYREF
  char v10; // [sp+28h] [bp-18h]

  sub_1AF68(a2, *(_DWORD **)a2, *(_DWORD *)(a2 + 4));
  v4 = (unsigned __int8 *)a1[41];
  while ( 1 )
  {
    result = (unsigned __int8 *)a1[42];
    if ( v4 == result )
      break;
    v6 = sub_173C4(a1[6] + 652, *v4);
    v7 = *v4++;
    v10 = v7;
    v9[0] = *(_DWORD *)(v6 + 8);
    v9[1] = *(_DWORD *)(v6 + 20);
    v9[2] = sub_4DF4C(*(_DWORD *)(a1[6] + 656), *(unsigned __int8 *)(v6 + 64));
    v8 = *(unsigned __int8 *)(v6 + 60)
       | (*(unsigned __int8 *)(v6 + 61) << 8)
       | (*(unsigned __int8 *)(v6 + 62) << 16)
       | (*(unsigned __int8 *)(v6 + 63) << 24);
    v9[3] = (unsigned __int16)(*(unsigned __int8 *)(v6 + 60) | (*(unsigned __int8 *)(v6 + 61) << 8));
    v9[5] = HIWORD(v8);
    v9[6] = HIWORD(v8) + 40;
    v9[4] = (unsigned __int16)v8 + 50;
    v9[7] = *(_DWORD *)(v6 + 52);
    sub_1AC70(a2, v9);
  }
  return result;
}

