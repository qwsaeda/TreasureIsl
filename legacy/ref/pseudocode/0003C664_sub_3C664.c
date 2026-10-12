/* ea:   0x0003C664 */
/* name: sub_3C664 */
/* size: 256 */
int __fastcall sub_3C664(int a1, int a2, int a3, int a4)
{
  int v7; // r4
  int v8; // r6
  int v9; // r0
  int v10; // r0
  unsigned int v11; // r0
  char v13; // [sp+0h] [bp-1Ch] BYREF

  v7 = (unsigned __int8)(rand() % 26 + 1);
  if ( v7 == 4 || v7 == 13 || v7 == 14 || v7 == 15 )
    v7 = (unsigned __int8)(v7 + 4);
  v8 = sub_173C4(a4 + 652, v7);
  v9 = sub_3AA5C(a3, a4);
  v13 = v7;
  sub_161DC(v9 + 164, &v13);
  if ( (_DWORD *)(a1 + 36) != dword_81184 )
    sub_17890((int)dword_81184, *(__int16 **)(a1 + 36), *(__int16 **)(a1 + 40));
  v10 = sub_12F04(aC);
  v11 = sub_3B0C0(dword_81184, (int)aC, 0, v10);
  return sub_3AF0C(dword_81184, v11, 2, v8 + 8);
}

