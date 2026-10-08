/* ea:   0x0003C094 */
/* name: sub_3C094 */
/* size: 208 */
int __fastcall sub_3C094(int a1, int a2, __int16 a3, _DWORD *a4)
{
  _DWORD *i; // r4
  int v8; // r2
  char v10; // [sp+0h] [bp-30h] BYREF
  _BYTE v11[40]; // [sp+8h] [bp-28h] BYREF

  sub_3CDA0(v11, &v10);
  for ( i = *(_DWORD **)(a4[127] + 8); i != (_DWORD *)a4[127]; i = sub_16A30(i) )
  {
    v8 = i[5];
    if ( *(unsigned __int8 *)(v8 + 50) == a3 && *(unsigned __int8 *)(v8 + 49) > 1u )
    {
      sub_1822C(i[5], a4 + 8);
      sub_34B38(a4, *((unsigned __int16 *)i + 8), 2);
      break;
    }
  }
  if ( (_DWORD *)(a1 + 36) != dword_81184 )
    sub_17890((int)dword_81184, *(__int16 **)(a1 + 36), *(__int16 **)(a1 + 40));
  return sub_3CDB4(v11);
}

