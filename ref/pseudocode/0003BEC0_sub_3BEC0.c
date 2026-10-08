/* ea:   0x0003BEC0 */
/* name: sub_3BEC0 */
/* size: 240 */
int __fastcall sub_3BEC0(int a1, int a2, int a3, _DWORD *a4)
{
  _DWORD *i; // r5
  int v8; // r0
  unsigned __int16 *v9; // r5
  int v10; // t1
  int v12; // r0
  char v13; // [sp+0h] [bp-30h] BYREF
  _BYTE v14[40]; // [sp+8h] [bp-28h] BYREF

  sub_3CDA0(v14, &v13);
  for ( i = *(_DWORD **)(a4[127] + 8); ; i = sub_16A30(i) )
  {
    if ( i == (_DWORD *)a4[127] )
    {
      v12 = sub_12F04(aAllLandHasOwne);
      sub_1318C((void **)dword_81184, (char *)aAllLandHasOwne, (int)&aAllLandHasOwne[v12]);
      return sub_3CDB4(v14);
    }
    if ( *(_BYTE *)(i[5] + 50) == 0 )
      break;
  }
  v8 = sub_3AA5C(a3, (int)a4);
  v10 = *((unsigned __int16 *)i + 8);
  v9 = (unsigned __int16 *)(i + 4);
  sub_513CC(v8, v10, 5);
  if ( (_DWORD *)(a1 + 36) != dword_81184 )
    sub_17890((int)dword_81184, *(__int16 **)(a1 + 36), *(__int16 **)(a1 + 40));
  sub_34B38(a4, *v9, 2);
  return sub_3CDB4(v14);
}

