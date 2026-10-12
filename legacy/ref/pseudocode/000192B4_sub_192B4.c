/* ea:   0x000192B4 */
/* name: sub_192B4 */
/* size: 380 */
int __fastcall sub_192B4(int a1, int a2, int a3, int a4, int a5)
{
  int v8; // r7
  int v10; // r0
  int v11; // r5
  int v12; // r4
  int v13; // r0
  int v14; // r4
  int v15; // [sp+0h] [bp-4Ch] BYREF
  void *v16[4]; // [sp+8h] [bp-44h] BYREF
  int (__fastcall **v17)(void *); // [sp+18h] [bp-34h] BYREF
  char v18; // [sp+1Ch] [bp-30h]
  char v19; // [sp+24h] [bp-28h]

  if ( (unsigned __int8)sub_5F498(a2, a3, 3565, &v15) == 0 )
    return 0;
  v8 = (unsigned __int8)v15;
  sub_1333C(v16);
  if ( (unsigned __int8)sub_5F55C(a2, a3, 3783584, v16) == 0 )
  {
    sub_132A0(v16);
    return 0;
  }
  v10 = (*(int (__fastcall **)(int, void *, _DWORD, _DWORD))(*(_DWORD *)a5 + 8))(a5, v16[0], 0, 0);
  v11 = v10;
  if ( v10 == 0 )
  {
    sub_12EA0((int)v16);
    return 0;
  }
  v19 = 0;
  v17 = &off_6C938;
  v18 = 0;
  v12 = (*(int (__fastcall **)(int))(*(_DWORD *)v10 + 4))(v10);
  v13 = (*(int (__fastcall **)(int))(*(_DWORD *)v11 + 12))(v11);
  v14 = sub_645B4(&v17, v13, v12);
  (*(void (__fastcall **)(int, int))(*(_DWORD *)a5 + 12))(a5, v11);
  if ( v14 != 0 )
  {
    sub_19B64(a4, v8, v14);
    v17 = &off_6C910;
    sub_12F48(v16);
    return 1;
  }
  else
  {
    v17 = &off_6C910;
    sub_12EA0((int)v16);
    return 0;
  }
}

