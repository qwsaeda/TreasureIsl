/* ea:   0x0006740C */
/* name: sub_6740C */
/* size: 236 */
int *__fastcall sub_6740C(int a1, int a2, int a3)
{
  int *v6; // r5
  int v7; // r0
  bool v8; // zf
  int v9; // r0
  int v11; // r0

  v6 = (int *)(*(int (__fastcall **)(_DWORD, int, int))(a1 + 32))(*(_DWORD *)(a1 + 40), 1, 64);
  if ( v6 == nullptr )
    return nullptr;
  v7 = (*(int (__fastcall **)(_DWORD, int, int))(a1 + 32))(*(_DWORD *)(a1 + 40), 8, 1440);
  v8 = v7 == 0;
  v6[9] = v7;
  v9 = *(_DWORD *)(a1 + 40);
  if ( v8 )
  {
    (*(void (__fastcall **)(int, int *))(a1 + 36))(v9, v6);
    return nullptr;
  }
  v11 = (*(int (__fastcall **)(int, int, int))(a1 + 32))(v9, 1, a3);
  v6[10] = v11;
  if ( v11 == 0 )
  {
    (*(void (__fastcall **)(_DWORD, int))(a1 + 36))(*(_DWORD *)(a1 + 40), v6[9]);
    (*(void (__fastcall **)(_DWORD, int *))(a1 + 36))(*(_DWORD *)(a1 + 40), v6);
    return nullptr;
  }
  v6[14] = a2;
  v6[11] = v11 + a3;
  *v6 = 0;
  sub_67370(v6, a1, nullptr);
  return v6;
}

