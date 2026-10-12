/* ea:   0x000428F0 */
/* name: sub_428F0 */
/* size: 28 */
int __fastcall sub_428F0(int result, int a2, int a3)
{
  bool v3; // zf

  v3 = a2 == 0;
  if ( a2 == 0 )
    v3 = a3 == 13;
  if ( v3 )
    return sub_3DC04(*(_DWORD *)(result + 4), 6);
  return result;
}

