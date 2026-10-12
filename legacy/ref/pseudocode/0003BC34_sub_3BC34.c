/* ea:   0x0003BC34 */
/* name: sub_3BC34 */
/* size: 464 */
int __fastcall sub_3BC34(int a1, int a2, __int16 a3, _DWORD *a4)
{
  char v7; // r7
  _DWORD *v8; // r0
  _DWORD *v9; // r5
  int v10; // r3
  int v11; // r4
  int v12; // r5
  int v13; // r4
  __int16 v14; // r3
  int v16; // r0
  _BYTE v17[4]; // [sp+4h] [bp-3Ch] BYREF
  _DWORD *i; // [sp+8h] [bp-38h] BYREF
  _DWORD v19[2]; // [sp+Ch] [bp-34h] BYREF
  int v20; // [sp+14h] [bp-2Ch] BYREF
  int v21; // [sp+18h] [bp-28h]

  sub_3CDA0(&v20, v17);
  v7 = 1;
  for ( i = (_DWORD *)*sub_1DDF0((int)(a4 + 127), v19); ; i = sub_16A30(v9) )
  {
    v8 = sub_1DE08(a4 + 127, v19);
    v9 = i;
    if ( i == (_DWORD *)*v8 )
      break;
    v10 = i[5];
    if ( *(_WORD *)(v10 + 22) != 0 && *(unsigned __int8 *)(v10 + 50) == a3 )
    {
      sub_3CC18(&v20, &i);
      v9 = i;
    }
  }
  if ( v20 != v21 )
    v7 = 0;
  if ( v7 != 0 )
  {
    v16 = sub_12F04(aNoLandHasSailo);
    sub_1318C((void **)dword_81184, (char *)aNoLandHasSailo, (int)&aNoLandHasSailo[v16]);
    return sub_3CDB4(&v20);
  }
  else
  {
    v11 = (v21 - v20) >> 2;
    v12 = 4 * (rand() % v11);
    v13 = *(_DWORD *)(*(_DWORD *)(v12 + v20) + 20);
    v14 = 999;
    if ( 2 * (unsigned int)*(unsigned __int16 *)(v13 + 22) <= 0x3E7 )
      v14 = 2 * *(_WORD *)(v13 + 22);
    *(_WORD *)(v13 + 22) = v14;
    sub_3AB44(
      a1,
      0,
      *(unsigned __int8 *)(*(_DWORD *)(*(_DWORD *)(v12 + v20) + 20) + 50),
      (int)a4,
      *(unsigned __int16 *)(*(_DWORD *)(*(_DWORD *)(v12 + v20) + 20) + 22) >> 1);
    sub_34B38(a4, *(unsigned __int16 *)(*(_DWORD *)(v12 + v20) + 16), 2);
    return sub_3CDB4(&v20);
  }
}

