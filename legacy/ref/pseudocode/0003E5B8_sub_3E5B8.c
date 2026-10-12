/* ea:   0x0003E5B8 */
/* name: sub_3E5B8 */
/* size: 60 */
void __fastcall sub_3E5B8(int a1, _DWORD *a2)
{
  _DWORD *v3; // r6
  _DWORD *v4; // r4

  v3 = a2;
  if ( a2 != nullptr )
  {
    do
    {
      sub_3E5B8(a1, v3[3]);
      v4 = (_DWORD *)v3[2];
      free(v3);
      v3 = v4;
    }
    while ( v4 != nullptr );
  }
}

