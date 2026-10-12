/* ea:   0x0003C4CC */
/* name: sub_3C4CC */
/* size: 280 */
int __fastcall sub_3C4CC(int a1, int a2, int a3, _DWORD *a4)
{
  unsigned __int16 v5; // r4
  int v7; // r6
  int v8; // r7
  int v9; // r0
  unsigned int v10; // r4
  int v11; // r0
  int v12; // r0
  unsigned int v13; // r0
  wchar_t v15[28]; // [sp+8h] [bp-38h] BYREF

  v5 = a3;
  sub_3AA5C(a3, (int)a4);
  v7 = v5;
  v8 = rand() % 6 + 1;
  sub_2F0E8(a4, v5, v8);
  if ( (_DWORD *)(a1 + 36) != dword_81184 )
    sub_17890((int)dword_81184, *(__int16 **)(a1 + 36), *(__int16 **)(a1 + 40));
  v9 = sub_12F04(aT);
  v10 = sub_3B0C0(dword_81184, (int)aT, 0, v9);
  v11 = sub_186F0((int)a4, v7);
  sub_3AF0C(dword_81184, v10, 2, v11);
  itow(v8, v15, 10);
  v12 = sub_12F04(aR);
  v13 = sub_3B0C0(dword_81184, (int)aR, 0, v12);
  return sub_3AFD8(dword_81184, v13, 2, v15);
}

