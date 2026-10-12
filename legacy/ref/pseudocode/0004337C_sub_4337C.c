/* ea:   0x0004337C */
/* name: sub_4337C */
/* size: 132 */
int __fastcall sub_4337C(_DWORD *a1, int a2, int a3)
{
  int v3; // r5
  int v4; // r3
  int v5; // r3
  int result; // r0

  v3 = a1[1];
  if ( v3 == 0 )
    __debugbreak();
  v4 = a1[3];
  if ( *(_DWORD *)(v3 + 4) + v4 + a2 == 0 || v4 + a2 >= 176 )
    goto LABEL_9;
  if ( v3 == 0 )
    __debugbreak();
  v5 = a1[4];
  if ( *(_DWORD *)(v3 + 8) + v5 + a3 == 0 || (LOBYTE(result) = 1, v5 + a3 >= 220) )
LABEL_9:
    LOBYTE(result) = 0;
  return (unsigned __int8)result;
}

