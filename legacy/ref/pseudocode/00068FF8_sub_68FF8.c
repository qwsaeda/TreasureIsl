/* ea:   0x00068FF8 */
/* name: sub_68FF8 */
/* size: 248 */
int __fastcall sub_68FF8(int a1, _DWORD *a2, int a3, int a4, int a5)
{
  int v9; // r0
  int v10; // r7
  int v12; // r0
  int v13; // r4
  void (__fastcall *v14)(_DWORD, int); // r2
  _DWORD v15[9]; // [sp+18h] [bp-24h] BYREF

  v15[0] = 0;
  v9 = (*(int (__fastcall **)(_DWORD, int, int))(a5 + 32))(*(_DWORD *)(a5 + 40), 19, 4);
  v10 = v9;
  if ( v9 == 0 )
    return -4;
  v12 = sub_690F0(a1, 19, 19, 0, 0, a3, a2, a4, v15, v9);
  v13 = v12;
  if ( v12 == -3 )
  {
    v14 = *(void (__fastcall **)(_DWORD, int))(a5 + 36);
    *(_DWORD *)(a5 + 24) = aOversubscribed;
    v14(*(_DWORD *)(a5 + 40), v10);
    return -3;
  }
  else
  {
    if ( v12 == -5 || *a2 == 0 )
    {
      v13 = -3;
      *(_DWORD *)(a5 + 24) = aIncompleteDyna;
    }
    (*(void (__fastcall **)(_DWORD, int))(a5 + 36))(*(_DWORD *)(a5 + 40), v10);
    return v13;
  }
}

