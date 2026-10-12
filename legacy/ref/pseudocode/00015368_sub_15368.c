/* ea:   0x00015368 */
/* name: sub_15368 */
/* size: 376 */
int __fastcall sub_15368(int a1, int a2, int a3, int a4)
{
  int v7; // r4
  int v8; // r5
  _DWORD *v9; // r0
  _DWORD *v11; // r0
  unsigned __int8 v12; // [sp+0h] [bp-2Ch] BYREF
  _BYTE v13[7]; // [sp+1h] [bp-2Bh] BYREF
  _DWORD v14[9]; // [sp+8h] [bp-24h] BYREF

  sub_16B48(a2 + 164, 16);
  v7 = sub_3AA5C(a3, a4);
  v12 = byte_6C820[rand() % 12];
  sub_18704(a4, &v12, v7);
  v8 = sub_17504(a4 + 672, v12);
  (**(void (__fastcall ***)(int, _DWORD, int, int))v8)(v8, 0, a3, a4);
  sub_16480(v14, &unk_7D7B4, v13);
  sub_162DC(v14, *(_DWORD *)(v8 + 8));
  sub_162DC(v14, &unk_7D7B0);
  v9 = (_DWORD *)sub_16AD8(v8, 1);
  sub_162DC(v14, *v9);
  sub_161B0(&dword_81184, a1 + 36);
  if ( (unsigned __int16)a3 == 1 )
  {
    sub_3AA9C(&dword_81184, aT, &unk_7D724);
  }
  else
  {
    v11 = (_DWORD *)sub_186F0(a4);
    sub_3AA9C(&dword_81184, aT, *v11);
  }
  sub_162DC(&dword_81184, v14[0]);
  return sub_12EA0((int)v14);
}

